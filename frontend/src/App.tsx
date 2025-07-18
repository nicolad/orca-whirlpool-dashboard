import { useState } from "react";
import { SignIn, SignedIn, SignedOut } from "@clerk/clerk-react";
import UsersTable from "./components/users-table";
import FilesTable from "./components/files-table";

function App() {
  const [text, setText] = useState("");

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    // Fire-and-forget: just call fetch with no awaits or .then handlers
    fetch("/api/speech", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ input: text }),
    });

    // (Optional) You can clear the text or otherwise give user feedback immediately:
    setText("");
  }

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <SignedOut>
        <SignIn />
      </SignedOut>
      <SignedIn>
        <div className="p-4 max-w-xl mx-auto">
          <h1 className="text-xl font-bold mb-4">
            Text-to-Speech Demo (Fire & Forget)
          </h1>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block mb-1 font-medium">
                Text to Speak (no response):
              </label>
              <textarea
                rows={5}
                value={text}
                onChange={(e) => setText(e.target.value)}
                className="border w-full p-2 text-black"
              />
            </div>
            <button
              type="submit"
              className="px-4 py-2 bg-blue-600 text-white rounded"
            >
              Send Request
            </button>
          </form>
        </div>
        <UsersTable />
        <FilesTable />
      </SignedIn>
    </main>
  );
}

export default App;
