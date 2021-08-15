use combine::{
    Parser, Stream,
    many1, error::{ParseError},
    parser::char::{char, spaces, letter, digit, string_cmp},
    optional, sep_end_by1, choice,
};

#[derive(Debug, PartialEq)]
pub struct Test {
    pub library: String,
    pub parameters: Vec<Param>,
}


#[derive(Debug, PartialEq)]
pub struct Param {
    pub arg_direction: ArgDirection,
    pub param_type: ParamType,
    pub name: String, 
    pub value: ValueType,
}

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Text { data: String },
    Number { data: f32, unit: Option<String>},
    Bool { data: bool },
}

#[derive(Debug, PartialEq)]
pub enum ParamType {
    Sigrefexpr,
    Voltage, 
    Current,
    String,
    Integer,
    Real,
    Time,
    Bool,
    Enum,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum ArgDirection {
    In,
    Out,
    Unknown,
}

fn number<Input>() -> impl Parser<Input, Output = ValueType>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        optional(char('-')).skip(spaces()),
        many1::<String, _, _>(digit()),
        optional(char('.').with(many1::<String, _, _>(digit()))),
        optional(many1(letter()).skip(spaces())),
    )
        .map(|(sign, int, dec, unit)| {
            let mut flt: String = String::new();
            match sign {
                Some('-') => flt.push('-'),
                Some(_) => {},
                None => {},
            };
            flt.push_str(&int);
            match dec {
                Some(ref a) => {
                    flt.push('.');
                    flt.push_str(&a);
                }
                None => {}
            }
            let result: f32 = flt.parse::<f32>().unwrap_or(f32::MIN);
            ValueType::Number{ data: result, unit: unit }
        })
}

fn text<Input>() -> impl Parser<Input, Output = ValueType>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(choice((letter(), digit(), char('_')))).skip(spaces()).map(|v| ValueType::Text{ data: v })
}

fn boolean<Input>() -> impl Parser<Input, Output = ValueType>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (choice((
        string_cmp("true", |l, r| l.eq_ignore_ascii_case(&r)).skip(spaces()),
        string_cmp("false", |l, r| l.eq_ignore_ascii_case(&r)).skip(spaces()),
    ))).map(|v| match v {
            "true" => ValueType::Bool{ data: true },
            _ => ValueType::Bool{ data: false },
        })
}

fn parameter<Input>() -> impl Parser<Input, Output = Param>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        spaces(),
        many1::<String, _, _>(letter()).skip(spaces()),
        many1::<String, _, _>(letter().or(char('_'))).skip(spaces()),
        many1::<String, _, _>(letter().or(char('_'))).skip(spaces()),
        char('=').skip(spaces()),
        choice((number(), boolean(), text())).skip(spaces()),
    )
        .map(|(_, arg, param, name, _, value)| Param {
            arg_direction: match arg.as_str() {
                "In" => ArgDirection::In,
                "Out" => ArgDirection::Out,
                _ => ArgDirection::Unknown,
            },
            param_type: match param.as_str() {
                "sigref_expr" => ParamType::Sigrefexpr,
                "Voltage" => ParamType::Voltage,
                "Current" => ParamType::Current,
                "String" => ParamType::String,
                "Integer" => ParamType::Integer,
                "Real" => ParamType::Real,
                "Time" => ParamType::Time,
                "Bool" => ParamType::Bool,
                "Enum" => ParamType::Enum,
                _ => ParamType::Unknown,
            },
            name: name,
            value: value,
        })
}

fn parameters<Input>() -> impl Parser<Input, Output = Vec<Param>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_end_by1(parameter().skip(spaces()), char(';').skip(spaces()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text() {
        let result1 = text().parse("L___Hmmm");
        let result2 = text().parse("fOobAR");

        assert_eq!(result1, Ok((ValueType::Text{ data: "L___Hmmm".to_string() }, "")));
        assert_eq!(result2, Ok((ValueType::Text{ data: "fOobAR".to_string() }, "")));
    }

    #[test]
    fn test_boolean() {
        let result1 = boolean().parse("true");
        let result2 = boolean().parse("false");
        let result3 = boolean().parse("tRuE");
        let result4 = boolean().parse("tue");

        assert_eq!(result1, Ok((ValueType::Bool{ data: true }, "")));
        assert_eq!(result2, Ok((ValueType::Bool{ data: false }, "")));
        assert_eq!(result3, Ok((ValueType::Bool{ data: true }, "")));
        assert!(result4.is_err());
    }

    #[test]
    fn test_number() {
        let result1 = number().parse("100V");
        let result2 = number().parse("-15.091mA");
        let result3 = number().parse("A");
        let result4 = number().parse("1534");

        assert_eq!(result1, Ok((ValueType::Number{ data: 100f32, unit: Some("V".to_string()) }, "")));
        assert_eq!(result2, Ok((ValueType::Number{ data: -15.091, unit: Some("mA".to_string()) }, "")));
        assert!(result3.is_err());
        assert_eq!(result4, Ok((ValueType::Number{ data: 1534f32, unit: None }, "")));
    }

    #[test]
    fn test_parse_parameter() {
        let result1 = parameter().parse("In Voltage Lower = -1.3V");
        let result2 = parameter().parse("In sigref_expr Pins = Y___MVN03");
        let result3 = parameter().parse("In Bool RequiredAWG = TRUE");
        let result4 = parameter().parse("      In Integer Dig_Length = 20");

        assert_eq!(result1, Ok((Param {
            arg_direction: ArgDirection::In,
            param_type: ParamType::Voltage,
            name: "Lower".to_string(),
            value: ValueType::Number{ data: -1.3, unit: Some("V".to_string()) },
        }, "")));
        assert_eq!(result2, Ok((Param {
            arg_direction: ArgDirection::In,
            param_type: ParamType::Sigrefexpr,
            name: "Pins".to_string(),
            value: ValueType::Text{ data: "Y___MVN03".to_string() },
        }, "")));
        assert_eq!(result3, Ok((Param {
            arg_direction: ArgDirection::In,
            param_type: ParamType::Bool,
            name: "RequiredAWG".to_string(),
            value: ValueType::Bool{ data: true },
        }, "")));
        assert_eq!(result4, Ok((Param {
            arg_direction: ArgDirection::In,
            param_type: ParamType::Integer,
            name: "Dig_Length".to_string(),
            value: ValueType::Number{ data: 20f32, unit: None },
        }, "")));
    }

    #[test]
    fn test_parse_parameters() {
        let params1 = "In Bool RequiredDig = TRUE;In Integer Dig_Length = 20;";
        let params2 = "    In Enum BoardType = ALL;
    In Time Interval = 100us;
    In Bool RequiredAWG = TRUE;
    In Integer AWG_StartAddr = 0;
    In Integer AWG_StopAddr = 19;
    In Integer AWG_ExecCount = 1;
    In Bool RequiredDig = TRUE;
    In Integer Dig_Length = 20;";
        let result1 = parameters().parse(params1);
        let result2 = parameters().parse(params2);
        assert_eq!(result1, Ok((vec![
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Bool,
                    name: "RequiredDig".to_string(), 
                    value: ValueType::Bool{ data: true },
                },
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Integer, 
                    name: "Dig_Length".to_string(), 
                    value: ValueType::Number{ data: 20f32, unit: None },
                },], "")));
        assert_eq!(result2, Ok((vec![
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Enum,
                    name: "BoardType".to_string(), 
                    value: ValueType::Text{ data: "ALL".to_string() },
                },
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Time, 
                    name: "Interval".to_string(), 
                    value: ValueType::Number{ data: 100f32, unit: Some("us".to_string()) },
                },
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Bool,
                    name: "RequiredAWG".to_string(), 
                    value: ValueType::Bool{ data: true },
                },
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Integer, 
                    name: "AWG_StartAddr".to_string(), 
                    value: ValueType::Number{ data: 0f32, unit: None },
                },
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Integer, 
                    name: "AWG_StopAddr".to_string(), 
                    value: ValueType::Number{ data: 19f32, unit: None },
                },
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Integer, 
                    name: "AWG_ExecCount".to_string(), 
                    value: ValueType::Number{ data: 1f32, unit: None },
                },
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Bool,
                    name: "RequiredDig".to_string(), 
                    value: ValueType::Bool{ data: true },
                },
                Param {
                    arg_direction: ArgDirection::In,
                    param_type: ParamType::Integer, 
                    name: "Dig_Length".to_string(), 
                    value: ValueType::Number{ data: 20f32, unit: None },
                },], "")));
    }
}
