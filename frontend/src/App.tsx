import { useState } from "react";
import UsersTable from "./components/users-table";
import FilesTable from "./components/files-table";
import { Box, Button, Flex, Heading, Text, TextArea } from "@radix-ui/themes";

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
    <Flex direction="column" align="center" gap="6" p="6" className="min-h-screen">
      <Box maxWidth="600px" width="100%">
        <Heading as="h1" size="6" mb="4">
          Text-to-Speech Demo (Fire & Forget)
        </Heading>
        <form onSubmit={handleSubmit}>
          <Flex direction="column" gap="3">
            <Text as="label" htmlFor="tts-input" weight="medium">
              Text to Speak (no response):
            </Text>
            <TextArea
              id="tts-input"
              rows={5}
              value={text}
              onChange={(e) => setText(e.target.value)}
            />
            <Button type="submit">Send Request</Button>
          </Flex>
        </form>
      </Box>
      <UsersTable />
      <FilesTable />
    </Flex>
  );
}

export default App;
