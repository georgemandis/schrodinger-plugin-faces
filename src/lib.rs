use schrodinger_plugin_sdk::prelude::*;
use schrodinger_plugin_sdk::schrodinger_plugin;

#[derive(Default)]
pub struct FacesPlugin;

impl NativePlugin for FacesPlugin {
    fn analyze(&self, entry: &ClipEntry) -> Result<AnalysisData, PluginError> {
        if !entry.has_image_format() {
            return Ok(AnalysisData::empty());
        }

        let image_data = entry.read_image()?;
        let tmp = tempfile::Builder::new()
            .suffix(".png")
            .tempfile()
            .map_err(|e| PluginError(format!("tempfile: {}", e)))?;
        std::fs::write(tmp.path(), &image_data)
            .map_err(|e| PluginError(format!("write: {}", e)))?;

        let faces = loupe_rs::detect_faces(tmp.path())
            .map_err(|e| PluginError(e))?;

        Ok(AnalysisData::new().set("face_count", faces.len()))
    }

    fn applies_to(&self, entry: &ClipEntry) -> bool {
        entry.has_image_format()
            && entry.analysis().has("face_count")
    }

    fn apply(
        &self,
        entry: &ClipEntry,
        mode: Option<&str>,
    ) -> Result<Vec<Output>, PluginError> {
        let image_data = entry.read_image()?;

        let input_tmp = tempfile::Builder::new()
            .suffix(".png")
            .tempfile()
            .map_err(|e| PluginError(format!("tempfile: {}", e)))?;
        std::fs::write(input_tmp.path(), &image_data)
            .map_err(|e| PluginError(format!("write: {}", e)))?;

        let output_tmp = tempfile::Builder::new()
            .suffix(".png")
            .tempfile()
            .map_err(|e| PluginError(format!("tempfile: {}", e)))?;

        let faces = loupe_rs::detect_faces(input_tmp.path())
            .map_err(|e| PluginError(e))?;

        if faces.is_empty() {
            return Ok(vec![Output::new("public.png").data(image_data)]);
        }

        match mode.unwrap_or("blur") {
            "redact" => loupe_rs::redact_faces(input_tmp.path(), output_tmp.path(), &faces),
            _ => loupe_rs::blur_faces(input_tmp.path(), output_tmp.path(), &faces),
        }.map_err(|e| PluginError(e))?;

        let result_data = std::fs::read(output_tmp.path())
            .map_err(|e| PluginError(format!("read result: {}", e)))?;

        Ok(vec![Output::new("public.png").data(result_data)])
    }
}

schrodinger_plugin!(FacesPlugin);
