# 📂 MIME Type Support Matrix

NetHop uses the `Content-Type` header from the server response to determine how to format and display data in the terminal. Below is the current support status for various media types.

## 📊 Support Table

| Status | MIME Type | Description | Pager Behavior |
| :---: | :--- | :--- | :--- |
| ✅ | `application/json` | Modern API responses | Automated Pretty-Printing via `serde_json` |
| ✅ | `text/plain` | Standard raw text | Direct passthrough to `less` |
| ✅ | `application/text` | Generic text | Direct passthrough to `less` |
| 🚧 | `text/html` | Webpage content | Future: Basic tag stripping or syntax highlighting |
| 🚧 | `application/xml` | Legacy API responses | Future: Tree-view indentation |
| 🚧 | `text/csv` | Spreadsheet data | Future: Column alignment and formatting |
| ❌ | `image/*` | Binary images | No plans for terminal rendering |
| ❌ | `application/pdf` | Document files | No plans for terminal rendering |

**Legend:**
- ✅ **Supported**: Fully implemented with formatting logic.
- 🚧 **In Development**: Planned for a future release (see [Roadmap](../README.md#roadmap)).
- ❌ **Not Supported**: Binary or incompatible formats.

#### [⬆ Back to README](../README.md)
