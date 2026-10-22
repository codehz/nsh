//! `nsh/clipboard`: the system clipboard's text, image and files
//! (UIPasteboard, ClipboardManager, NSPasteboard, `navigator.clipboard`,
//! ...). Needs the `clipboard` capability when the app declares
//! capabilities.

use crate::ui::{ClipboardImage, ClipboardImageBytes, ClipboardImageSource, module};

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

    /// Puts the image of `src` on the clipboard, replacing what it holds: a
    /// `blob:` URL, a base64 `data:` URI (as `readImage` gives), or a
    /// `Blob`. Nothing is put for a URL that names no live blob or a `Blob`
    /// whose bytes are no image.
    pub(crate) async fn write_image(src: ClipboardImageSource) {
        match src {
            ClipboardImageSource::Url(url) => crate::ui::prelude::set_clipboard_image(url),
            ClipboardImageSource::Blob { bytes, .. } => {
                let Some(mime) = crate::platform::clipboard::Image::sniff(&bytes)
                    .map(|image| image.mime.to_owned())
                else {
                    log::warn!(
                        target: "js",
                        "Clipboard.writeImage: the Blob's {} bytes are no image",
                        bytes.len()
                    );
                    return;
                };
                crate::ui::prelude::set_clipboard_image_bytes(bytes, mime);
            }
        }
    }

    /// The clipboard's image, `null` when it holds none: `url` is a
    /// base64 `data:` URI (an `Image` shows it and `fetch(url)` reads the
    /// bytes of), `size` its size in pixels; `null` too for an image over
    /// 48 MB, which the app's memory may not hold. On iOS, reading it may
    /// show the system's paste notice. Use `readImageBlob` for the bytes
    /// without the base64 form.
    pub(crate) async fn read_image() -> Option<ClipboardImage> {
        crate::ui::prelude::clipboard_image().await
    }

    /// The clipboard's image as bytes, `null` when it holds none: its
    /// encoded bytes (TS's `blob`), their type, and `size` in pixels;
    /// `null` too for an image over 48 MB. On iOS, reading it may show the
    /// system's paste notice.
    pub(crate) async fn read_image_blob() -> Option<ClipboardImageBytes> {
        crate::ui::prelude::clipboard_image_bytes().await
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
