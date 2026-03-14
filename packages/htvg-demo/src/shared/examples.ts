import type { HtvgDocument } from "htvg";

export interface ExampleDocument {
  name: string;
  doc: HtvgDocument;
}

export const examples: ExampleDocument[] = [
  {
    name: "hello",
    doc: {
      meta: { width: 400 },
      content: {
        type: "flex",
        style: {
          width: 400,
          padding: 32,
          backgroundColor: "#ffffff",
          flexDirection: "column",
          gap: 12,
        },
        children: [
          {
            type: "text",
            content: "Hello, HTVG!",
            style: { fontSize: 32, fontWeight: "bold", color: "#1a1a1a" },
          },
          {
            type: "text",
            content: "This is a self-contained document rendered to SVG.",
            style: { fontSize: 16, color: "#666666" },
          },
        ],
      },
    },
  },
  {
    name: "card",
    doc: {
      meta: { width: 360 },
      content: {
        type: "flex",
        style: {
          width: 360,
          padding: 24,
          backgroundColor: "#f8f9fa",
          flexDirection: "column",
          gap: 16,
        },
        children: [
          {
            type: "flex",
            style: {
              backgroundColor: "#ffffff",
              borderRadius: 12,
              borderWidth: 1,
              borderColor: "#e0e0e0",
              padding: 20,
              flexDirection: "column",
              gap: 8,
            },
            children: [
              {
                type: "text",
                content: "Card Title",
                style: { fontSize: 20, fontWeight: 700, color: "#1a1a1a" },
              },
              {
                type: "text",
                content:
                  "This is a card component with a border, rounded corners, and some padding. It demonstrates nested flex containers.",
                style: { fontSize: 14, color: "#555555", lineHeight: 1.5 },
              },
              {
                type: "flex",
                style: { flexDirection: "row", gap: 8, padding: "12 0 0 0" },
                children: [
                  {
                    type: "box",
                    style: {
                      width: 80,
                      height: 32,
                      backgroundColor: "#2563eb",
                      borderRadius: 6,
                    },
                  },
                  {
                    type: "box",
                    style: {
                      width: 80,
                      height: 32,
                      backgroundColor: "#e2e8f0",
                      borderRadius: 6,
                    },
                  },
                ],
              },
            ],
          },
          {
            type: "flex",
            style: {
              backgroundColor: "#ffffff",
              borderRadius: 12,
              borderWidth: 1,
              borderColor: "#e0e0e0",
              padding: 20,
              flexDirection: "column",
              gap: 8,
            },
            children: [
              {
                type: "text",
                content: "Another Card",
                style: { fontSize: 20, fontWeight: 700, color: "#1a1a1a" },
              },
              {
                type: "text",
                content: "Cards can be stacked vertically in a flex column layout.",
                style: { fontSize: 14, color: "#555555", lineHeight: 1.5 },
              },
            ],
          },
        ],
      },
    },
  },
  {
    name: "badge-row",
    doc: {
      meta: { width: 500 },
      content: {
        type: "flex",
        style: {
          width: 500,
          padding: 24,
          backgroundColor: "#1e293b",
          flexDirection: "column",
          gap: 16,
        },
        children: [
          {
            type: "text",
            content: "Status Dashboard",
            style: { fontSize: 24, fontWeight: "bold", color: "#f1f5f9" },
          },
          {
            type: "flex",
            style: { flexDirection: "row", gap: 10, flexWrap: "wrap" },
            children: [
              {
                type: "flex",
                style: {
                  backgroundColor: "#22c55e",
                  borderRadius: 16,
                  padding: "6 14",
                },
                children: [
                  {
                    type: "text",
                    content: "Passing",
                    style: { fontSize: 13, fontWeight: 600, color: "#ffffff" },
                  },
                ],
              },
              {
                type: "flex",
                style: {
                  backgroundColor: "#ef4444",
                  borderRadius: 16,
                  padding: "6 14",
                },
                children: [
                  {
                    type: "text",
                    content: "Failing",
                    style: { fontSize: 13, fontWeight: 600, color: "#ffffff" },
                  },
                ],
              },
              {
                type: "flex",
                style: {
                  backgroundColor: "#eab308",
                  borderRadius: 16,
                  padding: "6 14",
                },
                children: [
                  {
                    type: "text",
                    content: "Warning",
                    style: { fontSize: 13, fontWeight: 600, color: "#ffffff" },
                  },
                ],
              },
              {
                type: "flex",
                style: {
                  backgroundColor: "#3b82f6",
                  borderRadius: 16,
                  padding: "6 14",
                },
                children: [
                  {
                    type: "text",
                    content: "Info",
                    style: { fontSize: 13, fontWeight: 600, color: "#ffffff" },
                  },
                ],
              },
            ],
          },
          {
            type: "flex",
            style: {
              backgroundColor: "#334155",
              borderRadius: 8,
              padding: 16,
              flexDirection: "column",
              gap: 6,
            },
            children: [
              {
                type: "text",
                content: "Build #1247 completed in 3m 42s",
                style: { fontSize: 14, color: "#94a3b8" },
              },
              {
                type: "text",
                content: "All 128 tests passed. 0 failures.",
                style: { fontSize: 14, color: "#22c55e" },
              },
            ],
          },
        ],
      },
    },
  },
];

export const exampleMap = new Map(examples.map((example) => [example.name, example]));
