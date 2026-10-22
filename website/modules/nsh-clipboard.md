---
title: nsh/clipboard
description: "Clipboard: put text and images on the system clipboard, and read text, images and copied files back."
order: 8.5
---

# nsh/clipboard

The clipboard API reads and writes the system clipboard: text everywhere (`UIPasteboard` on iOS, `ClipboardManager` on Android, `NSPasteboard`, GDK and the Win32 clipboard on desktops, `navigator.clipboard` on the web), images on iOS, the desktops and the web, and the files copied in a file manager on the desktops and Mac Catalyst. Rust code calls the functions of `nsh_core::ui::prelude` (`set_clipboard`, `clipboard_string`, ...); TSX imports `Clipboard` from `nsh/clipboard`.

```rust tabs
use nsh_core::ui::prelude::*; // set_clipboard, clipboard_string, clipboard_image, ...
use nsh_core::ui::{ClipboardImage, ClipboardImageBytes};
```

```tsx tabs
import { Clipboard } from "nsh/clipboard";
```

```ts
const Clipboard: {
  writeText(text: string): Promise<void>;
  readText(): Promise<string>;
  hasText(): Promise<boolean>;
  writeImage(src: string | Blob): Promise<void>;
  readImage(): Promise<ClipboardImage | null>;
  readImageBlob(): Promise<ClipboardImageBlob | null>;
  hasImage(): Promise<boolean>;
  readFiles(): Promise<string[]>;
};

interface ClipboardImage {
  url: string; // data:image/…;base64,…
  size: { width: number; height: number }; // pixels
}

interface ClipboardImageBlob {
  blob: Blob;
  size: { width: number; height: number }; // pixels
}
```

| TS | Rust (`nsh_core::ui::prelude`) |
|---|---|
| `await Clipboard.writeText(text)` | `set_clipboard(text)` |
| `await Clipboard.readText()` | `clipboard_string().await -> String` |
| `await Clipboard.hasText()` | `clipboard_has_string().await -> bool` |
| `await Clipboard.writeImage(src)` | `set_clipboard_image(src)` or `set_clipboard_image_bytes(bytes, mime)` |
| `await Clipboard.readImage()` | `clipboard_image().await -> Option<ClipboardImage>` |
| `await Clipboard.readImageBlob()` | `clipboard_image_bytes().await -> Option<ClipboardImageBytes>` |
| `await Clipboard.hasImage()` | `clipboard_has_image().await -> bool` |
| `await Clipboard.readFiles()` | `clipboard_files().await -> Vec<String>` |

The Rust reads return answers to await in a task (`spawn`, or an `async` module function); `ClipboardImage` is `nsh_core::ui::ClipboardImage`, with `url` (a `data:` URI) and `size` (a `Size<DevicePixels>`). `ClipboardImageBytes` has the encoded `bytes`, their `mime`, and `size`; TSX sees them as a `ClipboardImageBlob` (`{ blob, size }`).

The user can also copy from a [selectable text](../elements/text.md) themselves; the clipboard API is for copying from code, such as a Copy button.

## Clipboard.writeText

```ts
writeText(text: string): Promise<void>;
```

Puts `text` on the clipboard, replacing what was there. Rust: `set_clipboard(text)`.

```rust tabs
use nsh_core::ui::prelude::*;

fn copy_button(code: Signal<String>) -> impl IntoElement {
    let copied = signal(false);
    view()
        .role(AccessibilityRole::Button)
        .on_press(move || {
            set_clipboard(code.get());
            copied.set(true);
        })
        .child(text(move || if copied.get() { "Copied" } else { "Copy" }))
}
```

```tsx tabs
import { signal, Text, View } from "nsh";
import { Clipboard } from "nsh/clipboard";

const copied = signal(false);

<View
  onPress={async () => {
    await Clipboard.writeText(code());
    copied.set(true);
  }}
>
  <Text>{copied() ? "Copied" : "Copy"}</Text>
</View>;
```

## Clipboard.readText

```ts
readText(): Promise<string>;
```

Resolves with the clipboard's text, or `""` when it holds none. Rust: `clipboard_string()`. On iOS, reading the clipboard can show the system's paste notice ("App pasted from …"), and on Android 10 and later only the app in the foreground can read it. Read it when the user asks to paste, not on every screen.

```rust tabs
spawn(async move {
    let pasted = clipboard_string().await;
    if !pasted.is_empty() {
        code.set(pasted.trim().to_string());
    }
});
```

```tsx tabs
const pasted = await Clipboard.readText();
if (pasted) code.set(pasted.trim());
```

## Clipboard.hasText

```ts
hasText(): Promise<boolean>;
```

Resolves with whether the clipboard holds text, without reading it: no paste notice on iOS. Rust: `clipboard_has_string()`. Use it to decide whether to show a Paste button. On the web, browsers only tell by reading, so it may ask the user for permission.

```rust tabs
let can_paste = signal(false);
spawn(async move { can_paste.set(clipboard_has_string().await) });
```

```tsx tabs
const canPaste = signal(false);
Clipboard.hasText().then((has) => canPaste.set(has));
```

## Clipboard.readImage

```ts
readImage(): Promise<ClipboardImage | null>;
```

Resolves with the clipboard's image, or `null` when it holds none: a pasted screenshot, or an image copied in a browser
or an image editor. Rust: `clipboard_image()`, `None` when it holds none. `url` is a base64 `data:` URI of the image
(PNG, or the clipboard's own JPEG, GIF, WebP or BMP), which an [`Image`](../elements/image.md) shows as its `src`; in
TSX `fetch(url)` gives its bytes. It owns its bytes: nothing has to be revoked. `size` (`width` and `height`) is its
size in pixels. As with `readText`, read it when the user asks to paste. Images of up to 48 MB are read; for a larger
one `readImage` resolves `null` and `hasImage` `true`.

For the bytes without the base64 string (an upload, a large image), use [`readImageBlob`](#clipboardreadimageblob).

```rust tabs
use nsh_core::ui::{ClipboardImage, prelude::*};

fn attachments() -> impl IntoElement {
    let attachments = signal(Vec::<ClipboardImage>::new());
    let paste = move || {
        spawn(async move {
            if let Some(image) = clipboard_image().await {
                attachments.update(|list| list.push(image));
            }
        });
    };
    view()
        .flex_row()
        .gap_2()
        .child(text("Paste").on_press(paste))
        .child(each(
            attachments,
            |shot: &ClipboardImage| shot.url.clone(),
            |shot, _| image(move || shot.with(|s| s.url.clone())).size(px(64.0)).rounded(px(8.0)),
        ))
}

```

```tsx tabs
import { Image, signal, View } from "nsh";
import { Clipboard, type ClipboardImage } from "nsh/clipboard";

const attachments = signal<ClipboardImage[]>([]);

async function paste() {
  const image = await Clipboard.readImage();
  if (image) attachments.set([...attachments(), image]);
}

<View class="flex-row gap-2">
  <Each of={attachments()}>
    {(image) => <Image src={image().url} style={{ width: 64, height: 64, borderRadius: 8 }} />}
  </Each>
</View>;
```

A chat-style app where users paste screenshots into a composer binds the paste shortcut while its input has the focus
and takes the image when there is one:

```rust tabs
use nsh_core::keymap::When;

on_key_with("mod+v", KeyOptions { when: When::Always, ..KeyOptions::default() }, move |_| {
    spawn(async move {
        if clipboard_has_image().await {
            paste().await;
        } else {
            insert_text(&clipboard_string().await);
        }
    });
});
```

```tsx tabs
import { keymap } from "nsh/keymap";

keymap.bind("mod+v", async () => {
  if (await Clipboard.hasImage()) return paste();
  insertText(await Clipboard.readText());
}, { when: "always" });
```

## Clipboard.readImageBlob

```ts
readImageBlob(): Promise<ClipboardImageBlob | null>;
```

Resolves with the clipboard's image as bytes, or `null` when it holds none: `blob` is a `Blob` of the image's MIME type
(PNG, or the clipboard's own JPEG, GIF, WebP or BMP), and `size` (`width` and `height`) is its size in pixels. Rust:
`clipboard_image_bytes()`, with `bytes`, `mime` and `size`. No base64 string is made and there is no URL to revoke; a
`Blob` owns its bytes. Images of up to 48 MB are read; for a larger one it resolves `null` and `hasImage` `true`. As
with `readText`, read it when the user asks to paste. To show the image, `URL.createObjectURL(image.blob)` and revoke
that URL when the view goes.

```rust tabs
use nsh_core::ui::prelude::*;

async fn upload_clipboard_image() -> Result<HttpResponse, String> {
    let image = clipboard_image_bytes().await.ok_or("no clipboard image")?;
    let request = HttpRequest::post("https://example.com/upload", image.bytes.to_vec())
        .header("Content-Type", image.mime);
    fetch(request).await
}
```

```tsx tabs
import { Clipboard } from "nsh/clipboard";

const image = await Clipboard.readImageBlob();
if (image) {
  const body = new FormData();
  body.append("file", image.blob, "screenshot.png");
  await fetch("https://example.com/upload", { method: "POST", body });
}
```

## Clipboard.hasImage

```ts
hasImage(): Promise<boolean>;
```

Resolves with whether the clipboard holds an image, without reading it: no paste notice on iOS. Rust:
`clipboard_has_image()`.

## Clipboard.writeImage

```ts
writeImage(src: string | Blob): Promise<void>;
```

Puts the image of `src` on the clipboard, replacing what it holds: a `blob:` URL, a base64 `data:` URL (as
`readImage` gives), or a `Blob` (as `readImageBlob` gives). A `Blob`'s bytes decide its type: they are sniffed, so a
`Blob` whose `type` is empty or differs still writes what it holds; bytes that are no image write nothing. Rust:
`set_clipboard_image(src)` for a URL, `set_clipboard_image_bytes(bytes, mime)` for bytes. On Windows it takes a PNG,
the format browsers, Office and the Snipping Tool read.

```rust tabs
set_clipboard_image(image.url.clone());
// Or the bytes: set_clipboard_image_bytes(image.bytes, image.mime);
```

```tsx tabs
await Clipboard.writeImage(image.url);
// Or: await Clipboard.writeImage(image.blob);
```

## Clipboard.readFiles

```ts
readFiles(): Promise<string[]>;
```

Resolves with the `file:` URLs of the files and folders copied in a file manager (Finder, Explorer, Files), in their
order, or `[]`: the same URLs a [file drop](../elements/view.md#desktop-file-drops) gives. Rust: `clipboard_files()`.
Parse a URL to get its path.

```rust tabs
let paths: Vec<std::path::PathBuf> = clipboard_files()
    .await
    .iter()
    .filter_map(|file| url::Url::parse(file).ok()?.to_file_path().ok())
    .collect();
```

```tsx tabs
const files = await Clipboard.readFiles();
const names = files.map((url) => decodeURIComponent(new URL(url).pathname.split("/").pop()!));
```

> [!NOTE]
> When your app declares `capabilities` in `nsh.json`, the clipboard must be granted with `"clipboard": true`. Without it `writeText` and `writeImage` do nothing, `readText` resolves `""`, `readImage` and `readImageBlob` `null`, `readFiles` `[]` and `hasText` and `hasImage` `false`, from Rust and TSX alike. See [Sandbox and capabilities](../docs/sandbox-and-capabilities.md).
