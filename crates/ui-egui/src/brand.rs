// SPDX-License-Identifier: MIT OR Apache-2.0
//! Luz's own mark: the app icon (`assets/app-icon/`), decoded once and kept as a texture, for About.

const ICON: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/256x256/apps/dev.danielmiguelt.Luz.png");

/// The app icon as a texture, or `None` when the embedded PNG does not decode.
pub fn mark(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::unique("luz-mark");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(t);
    }
    let mut reader = png::Decoder::new(std::io::Cursor::new(ICON)).read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => buf[..info.buffer_size()].as_chunks::<3>().0.iter().flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        _ => return None,
    };
    let color = egui::ColorImage::from_rgba_unmultiplied([info.width as usize, info.height as usize], &rgba);
    let tex = ctx.load_texture("luz-mark", color, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    Some(tex)
}

/// Paint the app icon over `r`.
pub fn paint_mark(painter: &egui::Painter, r: egui::Rect) {
    if let Some(tex) = mark(painter.ctx()) {
        painter.image(tex.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_icon_decodes() {
        let ctx = egui::Context::default();
        let tex = super::mark(&ctx).expect("the app icon decodes");
        assert_eq!(tex.size(), [256, 256]);
    }
}
