import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export function Markdown({ children }) {
  return (
    <div style={{ lineHeight: 1.6, wordBreak: "break-word" }}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h1: ({ node, ...props }) => (
            <h3 style={{ margin: "4px 0", fontWeight: 700 }} {...props} />
          ),
          h2: ({ node, ...props }) => (
            <h3 style={{ margin: "4px 0", fontWeight: 700 }} {...props} />
          ),
          h3: ({ node, ...props }) => (
            <h4 style={{ margin: "4px 0", fontWeight: 700 }} {...props} />
          ),
          p: ({ node, ...props }) => (
            <p style={{ margin: "4px 0" }} {...props} />
          ),
          ul: ({ node, ...props }) => (
            <ul style={{ margin: "4px 0", paddingLeft: "16px" }} {...props} />
          ),
          ol: ({ node, ...props }) => (
            <ol style={{ margin: "4px 0", paddingLeft: "16px" }} {...props} />
          ),
          li: ({ node, ...props }) => (
            <li style={{ margin: "2px 0" }} {...props} />
          ),
          strong: ({ node, ...props }) => (
            <strong style={{ fontWeight: 700 }} {...props} />
          ),
          table: ({ node, ...props }) => (
            <table
              style={{
                width: "100%",
                borderCollapse: "collapse",
                margin: "4px 0",
              }}
              {...props}
            />
          ),
          th: ({ node, ...props }) => (
            <th
              style={{
                border: "1px solid var(--sd-wood-light)",
                padding: "3px 6px",
                background: "var(--sd-parchment-d)",
                textAlign: "left",
              }}
              {...props}
            />
          ),
          td: ({ node, ...props }) => (
            <td
              style={{
                border: "1px solid var(--sd-wood-light)",
                padding: "3px 6px",
              }}
              {...props}
            />
          ),
          code: ({ node, ...props }) => (
            <code
              style={{
                background: "var(--sd-parchment-d)",
                padding: "1px 3px",
              }}
              {...props}
            />
          ),
          pre: ({ node, ...props }) => (
            <pre
              style={{
                background: "var(--sd-parchment-d)",
                padding: "6px",
                overflowX: "auto",
                margin: "4px 0",
              }}
              {...props}
            />
          ),
          blockquote: ({ node, ...props }) => (
            <blockquote
              style={{
                borderLeft: "3px solid var(--sd-wood-light)",
                paddingLeft: "8px",
                margin: "4px 0",
                color: "var(--sd-text-light)",
              }}
              {...props}
            />
          ),
          hr: ({ node, ...props }) => (
            <hr
              style={{
                border: "none",
                borderTop: "1px dashed var(--sd-wood-light)",
                margin: "6px 0",
              }}
              {...props}
            />
          ),
        }}
      >
        {children}
      </ReactMarkdown>
    </div>
  );
}
