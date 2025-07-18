/* eslint-disable @typescript-eslint/no-explicit-any */
"use client";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { UserButton } from "@clerk/clerk-react";
import { Loader as LucideLoader } from "lucide-react";
import useSWR from "swr";

// Data shape returned by GET /api/speech/files
export interface FinalFileSchema {
  timestamp: string;
  file_path: string;
  dir_name: string;
}

export default function FilesTable() {
  const { isLoading, data } = useSWR("/api/files", (url) =>
    fetch(url).then((res) => res.json())
  );

  // 2) Handle loading / error states
  if (isLoading) {
    return (
      <div className="flex items-center gap-2">
        <LucideLoader className="w-4 h-4 animate-spin" />
        <span>Loading files...</span>
      </div>
    );
  }

  // 3) If there are no files, show a friendly message
  if (!data || data.length === 0) {
    return (
      <>
        <div className="fixed top-6 right-6">
          <UserButton afterSignOutUrl="/" />
        </div>
        <p>No files found.</p>
      </>
    );
  }

  // 4) Otherwise, render them in a table
  return (
    <>
      <div className="fixed top-6 right-6">
        <UserButton afterSignOutUrl="/" />
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Timestamp</TableHead>
            <TableHead>Audio</TableHead>
            <TableHead>File</TableHead>
            <TableHead>Video</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {data.map((file: any, index: any) => (
            <TableRow key={index}>
              <TableCell>{file.timestamp}</TableCell>
              <TableCell>
                <audio controls>
                  <source src={file.file_path} type="audio/mpeg" />
                  Your browser does not support the audio element.
                </audio>
              </TableCell>
              <TableCell>
                <a
                  href={file.file_path}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-600 underline"
                >
                  final.mp3
                </a>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </>
  );
}
