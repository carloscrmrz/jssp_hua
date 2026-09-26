use super::Args;
use clap::Parser;

#[test]
fn defaults_match_expected_values_when_no_flags_are_given() {
    let args = Args::try_parse_from(["jssp"]).unwrap();

    assert_eq!(args.instance, None);
    assert_eq!(args.instance_path, None);
    assert_eq!(args.seed, None);
    assert_eq!(args.threads, None);
}

#[test]
fn parses_long_flags_over_defaults() {
    let args = Args::try_parse_from([
        "jssp",
        "--instance",
        "1,2,3",
        "--seed",
        "42",
        "--threads",
        "4",
    ])
    .unwrap();

    assert_eq!(args.instance, Some("1,2,3".to_string()));
    assert_eq!(args.seed, Some(42));
    assert_eq!(args.threads, Some(4));
}

#[test]
fn instance_path_accepts_its_short_flag() {
    let args = Args::try_parse_from(["jssp", "-p", "input.jssp"]).unwrap();

    assert_eq!(args.instance_path, Some("input.jssp".to_string()));
}

#[test]
fn rejects_a_non_numeric_seed() {
    let result = Args::try_parse_from(["jssp", "--seed", "not-a-number"]);

    assert!(result.is_err());
}
