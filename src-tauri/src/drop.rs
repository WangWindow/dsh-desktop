use std::{
    fs,
    path::{Path, PathBuf},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use tauri::WebviewWindow;

pub fn forward(window: &WebviewWindow, paths: &[PathBuf]) {
    for path in paths {
        let Some(mime_type) = image_mime(path) else {
            continue;
        };
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("Failed to read dropped file {}: {error}", path.display());
                continue;
            }
        };

        if let Err(error) = window.eval(build_drop_script(name, mime_type, &bytes)) {
            eprintln!("Failed to forward dropped file {}: {error}", path.display());
        }
    }
}

pub(crate) fn build_drop_script(name: &str, mime_type: &str, bytes: &[u8]) -> String {
    let encoded = STANDARD.encode(bytes);

    format!(
        r#"(() => {{
            const bytes = Uint8Array.from(atob({encoded:?}), byte => byte.charCodeAt(0));
            const file = new File([bytes], {name:?}, {{ type: {mime_type:?} }});
            const dataTransfer = new DataTransfer();
            dataTransfer.items.add(file);
            for (const type of ["dragenter", "dragover", "drop"]) {{
                document.dispatchEvent(new DragEvent(type, {{
                    bubbles: true,
                    cancelable: true,
                    dataTransfer,
                }}));
            }}
        }})();"#
    )
}

fn image_mime(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        _ => None,
    }
}
