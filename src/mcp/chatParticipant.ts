import * as vscode from "vscode";
import { DaemonManager } from "../daemon/DaemonManager";

export function registerChatParticipant(
  context: vscode.ExtensionContext,
  getDaemonManager: () => DaemonManager | null
): void {
  // Check if chat namespace and createChatParticipant are available
  // (Defensive check in case the host VSCode doesn't support it yet, though engines is ^1.91.0)
  if (typeof vscode.chat === "undefined" || typeof vscode.chat.createChatParticipant === "undefined") {
    console.warn("[comP] Chat Participant API is not supported in this version of VSCode.");
    return;
  }

  const handler: vscode.ChatRequestHandler = async (
    request: vscode.ChatRequest,
    _context: vscode.ChatContext,
    stream: vscode.ChatResponseStream,
    token: vscode.CancellationToken
  ) => {
    const dm = getDaemonManager();
    if (!dm || !dm.isRunning()) {
      stream.markdown("Error: comP daemon is not running. Please start it from the comP sidebar first.");
      return;
    }

    // 1. Process references (attached files)
    let compressedContext = "";
    const references = request.references || [];

    for (const ref of references) {
      if (ref.id === "vscode.file") {
        const fileUri = ref.value as vscode.Uri;
        try {
          stream.progress(`Compressing ${vscode.workspace.asRelativePath(fileUri)}...`);
          // Compress using Skeleton level (2) for maximum token savings
          const compressed = await dm.compressFile(fileUri.fsPath, 2);
          const relativePath = vscode.workspace.asRelativePath(fileUri);
          const ext = relativePath.split('.').pop() || "";
          
          compressedContext += `File: ${relativePath}\n\`\`\`${ext}\n${compressed}\n\`\`\`\n\n`;
        } catch (error) {
          // Fallback to raw file content on error
          try {
            const doc = await vscode.workspace.openTextDocument(fileUri);
            const rawContent = doc.getText();
            const relativePath = vscode.workspace.asRelativePath(fileUri);
            const ext = relativePath.split('.').pop() || "";
            compressedContext += `File: ${relativePath} (Raw - compression failed)\n\`\`\`${ext}\n${rawContent}\n\`\`\`\n\n`;
          } catch (readError) {
            console.error(`[comP] Failed to read file ${fileUri.fsPath}:`, readError);
          }
        }
      }
    }

    // 2. Build messages for the LLM
    const systemInstruction = 
      "You are comP, an AI context assistant. You are provided with compressed source code context (often with function bodies replaced by '{ ... }' to save tokens). " +
      "Analyze the skeleton of the code and respond to the user query. Do not complain about missing details in '{ ... }' unless you absolutely need to see them to answer.";

    const promptText = compressedContext
      ? `${systemInstruction}\n\nCompressed Context:\n\n${compressedContext}\n\nUser Request: ${request.prompt}`
      : `${systemInstruction}\n\nUser Request: ${request.prompt}`;

    const messages: vscode.LanguageModelChatMessage[] = [
      vscode.LanguageModelChatMessage.User(promptText)
    ];

    try {
      // 3. Send request to the user's selected language model
      const model = request.model;
      const chatResponse = await model.sendRequest(messages, {}, token);

      // 4. Stream response to VSCode chat UI
      for await (const fragment of chatResponse.text) {
        stream.markdown(fragment);
      }
    } catch (error) {
      stream.markdown(`\n\n*Error interacting with the language model: ${error instanceof Error ? error.message : String(error)}*`);
    }
  };

  const participant = vscode.chat.createChatParticipant("comp.chat", handler);
  participant.iconPath = vscode.Uri.file(
    pathJoin(context.extensionPath, "resources", "comp-icon.png")
  );

  context.subscriptions.push(participant);
}

function pathJoin(...parts: string[]): string {
  return parts.join("/").replace(/\/+/g, "/");
}
