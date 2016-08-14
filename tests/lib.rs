extern crate env_logger;
extern crate record_query;
extern crate serde_json;

mod js_doctest {

    use std::io;
    use std::str;

    use env_logger;
    use serde_json;

    use record_query;
    use record_query::query;
    use record_query::value;

    fn parse_json_stream(stream: &[u8]) -> Vec<serde_json::Value> {
        use std::io::Read;

        let cursor = io::Cursor::new(stream);
        let deserializer = serde_json::StreamDeserializer::new(cursor.bytes());
        deserializer.map(Result::unwrap).collect()
    }

    fn run_js_doctest(input: &str, query_str: &str, expected_output_str: &str) {
        let _ = env_logger::init();
        let mut actual_output_bytes = Vec::new();

        {
            let mut cursor = io::Cursor::new(input.as_bytes());
            let source = value::json::source(&mut cursor);
            let sink = value::json::sink(&mut actual_output_bytes);

            let query = query::Query::parse(&query_str).unwrap();
            record_query::run_query(&query, source, sink).unwrap();
        }

        let mut expected_output = parse_json_stream(expected_output_str.as_bytes());
        let mut actual_output = parse_json_stream(&actual_output_bytes);

        for v in &mut expected_output {
            normalize_json_numbers(v);
        }

        for v in &mut actual_output {
            normalize_json_numbers(v);
        }

        assert_eq!(expected_output, actual_output);
    }

    fn normalize_json_numbers(value: &mut serde_json::Value) {
        use serde_json::Value::*;
        match value {
            &mut I64(v) => *value = F64(v as f64),
            &mut U64(v) => *value = F64(v as f64),
            &mut Array(ref mut vs) => {
                for v in vs {
                    normalize_json_numbers(v);
                }
            },
            &mut Object(ref mut vs) => {
                for (_, v) in vs {
                    normalize_json_numbers(v);
                }
            }
            _ => (),
        }
    }

    macro_rules! js_doctest {
        ($id:ident, $input:expr, $process:expr, $args:expr, $output:expr) => {
            #[test]
            #[allow(non_snake_case)]
            fn $id() {
                run_js_doctest($input, concat!($process, " ", $args), $output)
            }
        }
    }

    include!(concat!(env!("OUT_DIR"), "/js_doctests.rs"));
}
