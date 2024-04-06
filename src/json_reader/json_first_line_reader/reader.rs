use crate::json_reader::bytes_of_array_reader::*;
use crate::json_reader::json_value::AsJsonSlice;
use rust_extensions::array_of_bytes_iterator::*;

use self::sync_reader::find_the_end_of_the_string;

use super::JsonKeyValue;

use super::super::JsonParseError;

pub struct JsonFirstLineReader<TArrayOfBytesIterator: ArrayOfBytesIterator> {
    raw: TArrayOfBytesIterator,
    had_init: bool,
}

impl<TArrayOfBytesIterator: ArrayOfBytesIterator> JsonFirstLineReader<TArrayOfBytesIterator> {
    pub fn new(raw: TArrayOfBytesIterator) -> Self {
        Self {
            raw,
            had_init: false,
        }
    }

    pub fn get_src_slice(&self) -> &[u8] {
        self.raw.get_src_slice()
    }

    fn init_if_requires(&mut self) -> Result<bool, JsonParseError> {
        if self.had_init {
            let token = sync_reader::skip_white_spaces_and_get_expected_token(
                &mut self.raw,
                ExpectedTokenJsonObjectSeparatorOrCloseBracket,
            )?;

            if token.value == crate::consts::CLOSE_BRACKET {
                return Ok(true);
            }
        } else {
            sync_reader::skip_white_spaces_and_get_expected_token(
                &mut self.raw,
                ExpectedOpenJsonObjectToken,
            )?;
            self.had_init = true;
        }

        Ok(false)
    }

    pub fn get_next(&mut self) -> Option<Result<JsonKeyValue, JsonParseError>> {
        match self.init_if_requires() {
            Ok(end_of_object) => {
                if end_of_object {
                    return None;
                }
            }

            Err(err) => return Some(Err(err)),
        }

        let key_start = match sync_reader::skip_white_spaces_and_peek_expected_token(
            &mut self.raw,
            ExpectedJsonObjectKeyStart,
        ) {
            Ok(next_value) => next_value.pos,
            Err(err) => return Some(Err(err)),
        };

        let key_end = match find_the_end_of_the_string(&mut self.raw) {
            Ok(next_value) => next_value.pos,
            Err(err) => return Some(Err(err)),
        };

        match sync_reader::skip_white_spaces_and_get_expected_token(
            &mut self.raw,
            ExpectedJsonObjectKeyValueSeparator,
        ) {
            Ok(next_value) => next_value,
            Err(err) => return Some(Err(err)),
        };

        let value_start = match sync_reader::skip_white_spaces_and_peek_expected_token(
            &mut self.raw,
            ExpectedJsonValueStart,
        ) {
            Ok(next_value) => next_value,
            Err(err) => return Some(Err(err)),
        };

        let value_end =
            match sync_reader::find_the_end_of_the_object_value(&mut self.raw, value_start.value) {
                Ok(pos) => pos,
                Err(err) => return Some(Err(err)),
            };

        return Some(Ok(JsonKeyValue {
            name_start: key_start,
            name_end: key_end + 1,
            value_start: value_start.pos,
            value_end: value_end,
        }));
    }
}

impl<TArrayOfBytesIterator: ArrayOfBytesIterator> AsJsonSlice
    for JsonFirstLineReader<TArrayOfBytesIterator>
{
    fn as_slice(&self, start_index: usize, end_index: usize) -> &[u8] {
        self.raw.get_src_slice()[start_index..end_index].as_ref()
    }
}

impl<'s> Into<JsonFirstLineReader<SliceIterator<'s>>> for &'s [u8] {
    fn into(self) -> JsonFirstLineReader<SliceIterator<'s>> {
        let slice_iterator = SliceIterator::new(self);
        JsonFirstLineReader::new(slice_iterator)
    }
}

impl<'s> Into<JsonFirstLineReader<SliceIterator<'s>>> for &'s str {
    fn into(self) -> JsonFirstLineReader<SliceIterator<'s>> {
        let slice_iterator = SliceIterator::new(self.as_bytes());
        JsonFirstLineReader::new(slice_iterator)
    }
}

#[cfg(test)]
mod tests {
    use rust_extensions::array_of_bytes_iterator::*;

    use super::*;

    #[test]
    pub fn test_simple_parse() {
        let src_data = "{\"name1\":\"123\", \"name2\":true,       \"name3\":null, \"name4\":0.12, \"name5\":{\"a\":\"b\"}}".as_bytes();

        let slice_iterator = SliceIterator::new(src_data);
        let mut parser = JsonFirstLineReader::new(slice_iterator);

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name1\"", item.get_raw_name(&parser).unwrap());
        assert_eq!("\"123\"", item.get_raw_value(&parser).unwrap());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name2\"", item.get_raw_name(&parser).unwrap());
        assert_eq!("true", item.get_raw_value(&parser).unwrap());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name3\"", item.get_raw_name(&parser).unwrap());
        assert_eq!("null", item.get_raw_value(&parser).unwrap());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name4\"", item.get_raw_name(&parser).unwrap());
        assert_eq!("0.12", item.get_raw_value(&parser).unwrap());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("\"name5\"", item.get_raw_name(&parser).unwrap());
        assert_eq!("{\"a\":\"b\"}", item.get_raw_value(&parser).unwrap());

        let item = parser.get_next();

        assert_eq!(true, item.is_none());
    }

    #[test]
    fn test_json_first_line() {
        let fist_line = r#"{"processId":"8269e2ac-fa3b-419a-8e65-1a606ba07942","sellAmount":0.4,"buyAmount":null,"sellAsset":"ETH","buyAsset":"USDT"}"#.as_bytes();

        let slice_iterator = SliceIterator::new(fist_line);

        let mut parser = JsonFirstLineReader::new(slice_iterator);

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("processId", item.get_name(&parser).unwrap().as_str());
        assert_eq!(
            "8269e2ac-fa3b-419a-8e65-1a606ba07942",
            item.get_value().as_str(&parser).unwrap().as_str()
        );

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("sellAmount", item.get_name(&parser).unwrap().as_str());
        assert_eq!("0.4", item.get_value().as_str(&parser).unwrap().as_str());

        let item = parser.get_next().unwrap().unwrap();

        assert_eq!("buyAmount", item.get_name(&parser).unwrap().as_str());

        let value = item.get_value();
        assert!(value.is_null(&parser));
    }

    #[test]
    pub fn test_with_array_inside_json_l_split() {
        let json = r###"{"names":[{"company_name": "Company \"A\""},
                {
                    "company_name": "Company \"B\""
                },
                {
                    "company_name": "Company \"C\""
                }
            ],
            "registered_address": "Addr",
            "retrieved_at": "2010-02-23"
        }"###
            .as_bytes();

        let slice_iterator = SliceIterator::new(json);

        let mut first_line_reader = JsonFirstLineReader::new(slice_iterator);

        while let Some(sub_json) = first_line_reader.get_next() {
            let sub_json = sub_json.unwrap();
            println!(
                "{}",
                sub_json.get_name(&first_line_reader).unwrap().as_str(),
            );
        }
    }

    #[test]
    pub fn read_first_line_with_empty_value() {
        let json = r###"{
            "": true,
            "AD": false,
            "CD": false,
            "DK": false,
            "HD": false,
            "Note:": true,
            "SI": false,
            "UT": false,
            "VÃ": false
        }"###
            .as_bytes();

        let slice_iterator = SliceIterator::new(json);

        let mut first_line_reader = JsonFirstLineReader::new(slice_iterator);

        while let Some(sub_json) = first_line_reader.get_next() {
            let sub_json = sub_json.unwrap();
            println!(
                "{}:{}",
                sub_json.get_raw_name(&first_line_reader).unwrap(),
                sub_json.get_raw_value(&first_line_reader).unwrap()
            );
        }
    }
}
