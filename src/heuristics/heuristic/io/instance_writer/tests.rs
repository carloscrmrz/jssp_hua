use super::InstanceWriter;
use chrono::{Local, TimeZone, Timelike};

#[test]
fn output_file_name_embeds_the_padded_index_and_the_formatted_timestamp() {
    let at = Local.with_ymd_and_hms(2024, 3, 7, 9, 5, 2).unwrap();
    let name = InstanceWriter::output_file_name(3, &at);

    assert!(name.starts_with("./results/output-0003-"));
    assert!(name.ends_with(".out"));
    assert!(name.contains(&at.format("%d%m-%Y-%H%M-%S").to_string()));
}

#[test]
fn output_file_name_includes_centiseconds_from_subsecond_millis() {
    let at = Local
        .with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
        .unwrap()
        .with_nanosecond(456_000_000)
        .unwrap();

    let name = InstanceWriter::output_file_name(0, &at);

    assert!(name.ends_with("45.out"));
}

#[test]
fn solutions_file_name_replaces_extension_with_sol() {
    assert_eq!(
        InstanceWriter::solutions_file_name("./results/output-0001.out"),
        "./results/output-0001.sol"
    );
}

#[test]
fn solutions_file_name_appends_extension_when_missing() {
    assert_eq!(InstanceWriter::solutions_file_name("output"), "output.sol");
}

// TODO: report/solutions formatting tests once the JSSP problem and solution exist.
