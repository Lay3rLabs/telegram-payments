import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: any) {
    console.error("ErrorBoundary caught an error:", error, errorInfo);

    // Try to display error in Telegram if available
    try {
      if (typeof window !== "undefined" && (window as any).Telegram?.WebApp) {
        const WebApp = (window as any).Telegram.WebApp;
        if (WebApp.showAlert) {
          WebApp.showAlert(
            `Error: ${error.message}\n\nCheck console for details`
          );
        }
      }
    } catch (e) {
      console.error("Failed to show error in Telegram:", e);
    }
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            padding: "20px",
            background: "#fee",
            border: "2px solid #c33",
            borderRadius: "8px",
            margin: "20px",
          }}
        >
          <h1 style={{ color: "#c33", margin: "0 0 10px 0" }}>
            Something went wrong
          </h1>
          <p
            style={{
              margin: "10px 0",
              fontFamily: "monospace",
              fontSize: "14px",
            }}
          >
            {this.state.error?.message}
          </p>
          <pre
            style={{
              background: "#f5f5f5",
              padding: "10px",
              borderRadius: "4px",
              overflow: "auto",
              fontSize: "12px",
            }}
          >
            {this.state.error?.stack}
          </pre>
          <button
            onClick={() => {
              this.setState({ hasError: false, error: undefined });
              window.location.reload();
            }}
            style={{
              marginTop: "10px",
              padding: "10px 20px",
              background: "#3390ec",
              color: "white",
              border: "none",
              borderRadius: "6px",
              cursor: "pointer",
            }}
          >
            Reload Page
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
