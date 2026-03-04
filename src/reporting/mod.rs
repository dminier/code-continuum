use crate::analysis::AnalysisReport;

pub fn write_report(report: &AnalysisReport) -> Result<(), std::io::Error> {
    let _ = std::fs::create_dir_all(".output");
    let file = std::fs::File::create(".output/report.json")?;
    serde_json::to_writer_pretty(std::io::BufWriter::new(file), report)
        .map_err(std::io::Error::other)?;
    Ok(())
}
