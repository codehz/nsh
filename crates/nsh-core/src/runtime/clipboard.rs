//! `nsh/clipboard`: the system clipboard's text, image and files
//! (UIPasteboard, ClipboardManager, NSPasteboard, `navigator.clipboard`,
//! ...). Needs the `clipboard` capability when the app declares
//! capabilities.

use crate::ui::{ClipboardImage, module};

/// The system clipboard's text, image and files.
#[module]
pub(crate) struct Clipboard;

#[module]
impl Clipboard {
    /// Puts `text` on the clipboard, replacing what it holds.
    pub(crate) async fn write_text(text: String) {
        crate::ui::prelude::set_clipboard(text);
    }

    /// The clipboard's text, `""` when it holds none. On iOS, reading it
    /// may show the system's paste notice.
    pub(crate) async fn read_text() -> String {
        crate::ui::prelude::clipboard_string().await
    }

    /// Whether the clipboard holds text, without reading it (no paste
    /// notice).
    pub(crate) async fn has_text() -> bool {
        crate::ui::prelude::clipboard_has_string().await
    }

    /// Puts the image of `src` (a `blob:` URL, as `readImage` gives, or a
    /// base64 `data:` URL) on the clipboard, replacing what it holds.
    /// Nothing is put for a URL that names no live blob.
    pub(crate) async fn write_image(src: String) {
        crate::ui::prelude::set_clipboard_image(src);
    }

    /// The clipboard's image, `null` when it holds none: `url` is a
    /// `blob:` URL in the engine's store (an `Image` shows it and
    /// `fetch(url)` reads the bytes of), `size` its size in pixels; `null`
    /// too for an image over 48 MB, which the app's memory may not hold. On
    /// iOS, reading it may show the system's paste notice.
    pub(crate) async fn read_image() -> Option<ClipboardImage> {
        crate::ui::prelude::clipboard_image().await
    }

    /// Whether the clipboard holds an image, without reading it (no paste
    /// notice).
    pub(crate) async fn has_image() -> bool {
        crate::ui::prelude::clipboard_has_image().await
    }

    /// The `file:` URLs of the local files on the clipboard (files copied in
    /// a file manager), `[]` when it holds none.
    pub(crate) async fn read_files() -> Vec<String> {
        crate::ui::prelude::clipboard_files().await
    }
}
