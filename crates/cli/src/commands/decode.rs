//! `prism decode` — Decode a transaction error into plain English.

use clap::Args;
use prism_core::types::config::NetworkConfig;
use prism_core::types::report::{DiagnosticReport, Severity};

/// Arguments for the decode command.
#[derive(Args)]
pub struct DecodeArgs {
    /// Transaction hash to decode, or a raw error string with --raw.
    pub tx_hash: String,

    /// Decode a raw error string instead of fetching by TX hash.
    #[arg(long)]
    pub raw: bool,

    /// Show short one-line summary only.
    #[arg(long)]
    pub short: bool,
}

/// Execute the decode command.
pub async fn run(
    args: DecodeArgs,
    network: &NetworkConfig,
    output_format: &str,
) -> anyhow::Result<()> {
    if args.raw {
        let report = build_raw_xdr_report(&args.tx_hash)?;
        print_report(&report, output_format)?;
        return Ok(());
    }

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!(
        "Fetching transaction {}...",
        &args.tx_hash[..8.min(args.tx_hash.len())]
    ));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let report = prism_core::decode::decode_transaction(&args.tx_hash, network).await?;

    spinner.finish_and_clear();

    print_report(&report, output_format)?;

    Ok(())
}

fn build_raw_xdr_report(raw_xdr: &str) -> anyhow::Result<DiagnosticReport> {
    let bytes = prism_core::xdr::codec::decode_xdr_base64(raw_xdr)?;
    let mut report =
        DiagnosticReport::new("raw-xdr", 0, "RawXdr", "Decoded raw XDR input from --raw");
    report.severity = Severity::Info;
    report.detailed_explanation = format!(
        "Decoded {} bytes from the raw base64 XDR string provided on the command line.",
        bytes.len()
    );
    Ok(report)
}

fn print_report(report: &DiagnosticReport, output_format: &str) -> anyhow::Result<()> {
    match output_format {
        "json" => crate::output::json::print_report(report)?,
        "compact" => crate::output::compact::print_report(report)?,
        _ => crate::output::human::print_report(report)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_raw_xdr_report;

    #[test]
    fn raw_xdr_input_builds_a_local_report() {
        let report = build_raw_xdr_report("AAAA").expect("raw XDR should decode");

        assert_eq!(report.error_category, "raw-xdr");
        assert_eq!(report.error_name, "RawXdr");
        assert_eq!(report.summary, "Decoded raw XDR input from --raw");
        assert!(report.detailed_explanation.contains("3 bytes"));
    }
}
