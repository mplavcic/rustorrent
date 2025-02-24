pub fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let decoded: serde_bencode::value::Value =
        serde_bencode::from_bytes(encoded_value.as_bytes()).expect("Parsing Bencode");
    match decoded {
        // Integers are represented by an 'i' followed by the number in base 10 followed by an 'e'.
        // For example i3e corresponds to 3 and i-3e corresponds to -3. Integers have no size limitation.
        // i-0e is invalid. All encodings with a leading zero, such as i03e, are invalid, other than i0e,
        // which of course corresponds to 0.
        serde_bencode::value::Value::Int(number) => {
            return serde_json::Value::Number(number.into());
        }
        // Strings are length-prefixed base ten followed by a colon and the string.
        // For example 4:spam corresponds to 'spam'.
        serde_bencode::value::Value::Bytes(byte_vec) => {
            return serde_json::Value::String(
                String::from_utf8(byte_vec).expect("Invalid UTF-8 string"),
            );
        }
        // Lists are encoded as an 'l' followed by their elements (also bencoded) followed by an 'e'.
        // For example l4:spam4:eggse corresponds to ['spam', 'eggs'].
        serde_bencode::value::Value::List(bencode_value_vec) => {
            let json_list: Vec<serde_json::Value> = bencode_value_vec
                .into_iter()
                .map(|item| {
                    decode_bencoded_value(
                        &serde_bencode::to_string(&item).expect("Encoding Bencode"),
                    )
                })
                .collect();
            return serde_json::Value::Array(json_list);
        }
        // Dictionaries are encoded as a 'd' followed by a list of alternating keys and their corresponding
        // values followed by an 'e'. For example, d3:cow3:moo4:spam4:eggse corresponds to {'cow': 'moo', 'spam': 'eggs'}
        // and d4:spaml1:a1:bee corresponds to {'spam': ['a', 'b']}. Keys must be strings and appear in sorted order
        // (sorted as raw strings, not alphanumerics).
        serde_bencode::value::Value::Dict(bencode_map) => {
            let mut json_map = serde_json::Map::new();
            for (key, value) in bencode_map {
                let key_string = String::from_utf8(key).expect("Invalid UTF-8 dictionary key");
                let json_value = decode_bencoded_value(
                    &serde_bencode::to_string(&value).expect("Encoding Bencode"),
                );
                json_map.insert(key_string, json_value);
            }
            return serde_json::Value::Object(json_map);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /******** String ********/
    #[test]
    fn decode_bencoded_value_string() {
        let encoded = "5:hello";
        let expected = json!("hello");
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_string_empty() {
        let encoded = "0:";
        let expected = json!("");
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    #[should_panic]
    fn decode_bencoded_value_string_invalid_no_colon() {
        let encoded = "5hello";
        decode_bencoded_value(encoded);
    }

    #[test]
    #[should_panic]
    fn decode_bencoded_value_string_invalid_missing_length() {
        let encoded = "hello";
        decode_bencoded_value(encoded);
    }

    #[test]
    #[should_panic]
    fn decode_bencoded_value_string_invalid_bigger_length() {
        let encoded = "6:hello";
        decode_bencoded_value(encoded);
    }

    /******** Integer ********/
    #[test]
    fn decode_bencoded_value_integer() {
        let encoded = "i42e";
        let expected = json!(42);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_integer_negative() {
        let encoded = "i-42e";
        let expected = json!(-42);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_integer_zero() {
        let encoded = "i0e";
        let expected = json!(0);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    /*
    TODO: bencode encoding doesn't allow leading zero(s)
    #[test]
    #[should_panic]
    fn decode_bencoded_value_integer_leading_zero() {
        let encoded = "i042e";
        decode_bencoded_value(encoded);
    }

    #[test]
    #[should_panic]
    fn decode_bencoded_value_integer_multiple_zeros() {
        let encoded = "i00e";
        decode_bencoded_value(encoded);
    }
    */

    #[test]
    #[should_panic]
    fn decode_bencoded_value_integer_unterminated() {
        let encoded = "i42";
        decode_bencoded_value(encoded);
    }

    #[test]
    #[should_panic]
    fn decode_bencoded_value_integer_missing_start() {
        let encoded = "42e";
        decode_bencoded_value(encoded);
    }

    /******** List ********/
    #[test]
    fn decode_bencoded_value_list() {
        let encoded = "l5:hello2:woi42ee";
        let expected = json!(["hello", "wo", 42]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_list_empty() {
        let encoded = "le";
        let expected = json!([]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_list_nested() {
        let encoded = "l5:helloi10el3:fooi20eee";
        let expected = json!(["hello", 10, ["foo", 20]]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_list_deeply_nested() {
        let encoded = "ll3:oneei2ell3:abc3:xyzeli77eeee";
        let expected = json!([["one"], 2, [["abc", "xyz"], [77]]]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    #[should_panic]
    fn decode_bencoded_value_list_unterminated() {
        let encoded = "l3:fooi42e";
        decode_bencoded_value(encoded);
    }

    #[test]
    fn decode_bencoded_value_list_with_dict() {
        let encoded = "ld3:key5:value3:numi42eee";
        let expected = json!([
            {
                "key": "value",
                "num": 42
            }
        ]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_list_mixed_types() {
        let encoded = "l3:onei2ed3:key3:val3:numi10ee5:applee";
        let expected = json!([
            "one",
            2,
            {
                "key": "val",
                "num": 10
            },
            "apple"
        ]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_list_with_multiple_dicts() {
        let encoded = "ld3:foo3:bar3:numi100eed4:spam4:eggs5:spam24:milkee";
        let expected = json!([
            {
                "foo": "bar",
                "num": 100
            },
            {
                "spam": "eggs",
                "spam2": "milk"
            }
        ]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_list_nested_dicts() {
        let encoded = "ll3:one3:twoed3:foo3:bar3:numi5eeli9eee";
        let expected = json!([
            ["one", "two"],
            {
                "foo": "bar",
                "num": 5
            },
            [9]
        ]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_list_deeply_nested_dicts_and_lists() {
        let encoded = "ll5:apple6:bananaed3:keyd4:deepi99ee3:numi42eee";
        let expected = json!([
            ["apple", "banana"],
            {
                "key": {
                    "deep": 99
                },
                "num": 42
            }
        ]);
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    /******** Dictionary ********/
    #[test]
    fn decode_bencoded_value_dict_empty() {
        let encoded = "de";
        let expected = json!({});
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    #[should_panic]
    fn decode_bencoded_value_dict_unterminated() {
        let encoded = "d3:key5:value";
        decode_bencoded_value(encoded);
    }

    #[test]
    fn decode_bencoded_value_dict_single_pair() {
        let encoded = "d3:key5:valuee";
        let expected = json!({"key": "value"});
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_dict_multiple_pairs() {
        let encoded = "d3:foo3:bar3:baz3:quxe";
        let expected = json!({"foo": "bar", "baz": "qux"});
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_dict_with_list() {
        let encoded = "d3:fooi1e3:bari3e3:bazl5:hello5:worldee";
        let expected = json!({
            "foo": 1,
            "bar": 3,
            "baz": ["hello", "world"]
        });
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_dict_with_nested_dict() {
        let encoded = "d3:foo3:bar3:bazd3:key5:valueee";
        let expected = json!({
            "foo": "bar",
            "baz": {
                "key": "value"
            }
        });
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_dict_deeply_nested() {
        let encoded = "d3:foo3:bar3:bazd3:keyd4:deep4:nesteeee";
        let expected = json!({
            "foo": "bar",
            "baz": {
                "key": {
                    "deep": "nest"
                }
            }
        });
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_dict_with_mixed_values() {
        let encoded = "d3:numi42e3:str6:string3:lstli1ei2ei3ee4:dictd3:key3:valee";
        let expected = json!({
            "num": 42,
            "str": "string",
            "lst": [1, 2, 3],
            "dict": {
                "key": "val"
            }
        });
        assert_eq!(decode_bencoded_value(encoded), expected);
    }

    #[test]
    fn decode_bencoded_value_dict_large_complex_structure() {
        let encoded = "d4:spaml5:apple6:bananae3:numi100e4:metad4:infod4:auth4:John4:yeari2024ee4:tagsl4:rust3:ioseee";
        let expected = json!({
            "spam": ["apple", "banana"],
            "num": 100,
            "meta": {
                "info": {
                    "auth": "John",
                    "year": 2024
                },
                "tags": ["rust", "ios"]
            }
        });
        assert_eq!(decode_bencoded_value(encoded), expected);
    }
}
