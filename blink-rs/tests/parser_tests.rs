use blink::lexer::{Lexer, LexerMode, TokenType, TokenValue};
use blink::parser::{DeclarationType, Parser, TypeNodeValue};

// ---- Lexer Tests ----

#[test]
fn test_lexer_simple_tokens() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("{ } , = : < > ..");

    let expected = vec![
        TokenType::OpenBraces,
        TokenType::CloseBraces,
        TokenType::Comma,
        TokenType::Assign,
        TokenType::FieldAssign,
        TokenType::OpenChevrons,
        TokenType::CloseChevrons,
        TokenType::Merge,
        TokenType::EndOfFile,
    ];

    for expected_type in expected {
        let token = lexer.get_next_token(false, None);
        assert_eq!(
            token.token_type, expected_type,
            "Expected {:?}",
            expected_type
        );
    }
}

#[test]
fn test_lexer_keywords() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("type enum struct event function scope option export map set");

    for _ in 0..10 {
        let token = lexer.get_next_token(false, None);
        assert_eq!(token.token_type, TokenType::Keyword);
    }
}

#[test]
fn test_lexer_primitives() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("u8 u16 u32 i8 i16 i32 f16 f32 f64 boolean string vector buffer");

    for _ in 0..13 {
        let token = lexer.get_next_token(false, None);
        assert_eq!(token.token_type, TokenType::Primitive);
    }
}

#[test]
fn test_lexer_identifiers() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("MyType SomeStruct _private");

    for _ in 0..3 {
        let token = lexer.get_next_token(false, None);
        assert_eq!(token.token_type, TokenType::Identifier);
    }
}

#[test]
fn test_lexer_dotted_identifier() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("Scope.Type");

    let token = lexer.get_next_token(false, None);
    assert_eq!(token.token_type, TokenType::Identifier);
    assert_eq!(token.value, TokenValue::Str("Scope.Type".to_string()));
}

#[test]
fn test_lexer_strings() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize(r#""hello" "world""#);

    let token1 = lexer.get_next_token(false, None);
    assert_eq!(token1.token_type, TokenType::String);
    assert_eq!(token1.value, TokenValue::Str("hello".to_string()));

    let token2 = lexer.get_next_token(false, None);
    assert_eq!(token2.token_type, TokenType::String);
    assert_eq!(token2.value, TokenValue::Str("world".to_string()));
}

#[test]
fn test_lexer_booleans() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("true false");

    let token1 = lexer.get_next_token(false, None);
    assert_eq!(token1.token_type, TokenType::Boolean);
    assert_eq!(token1.value, TokenValue::Bool(true));

    let token2 = lexer.get_next_token(false, None);
    assert_eq!(token2.token_type, TokenType::Boolean);
    assert_eq!(token2.value, TokenValue::Bool(false));
}

#[test]
fn test_lexer_import_as() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("import as");

    let t1 = lexer.get_next_token(false, None);
    assert_eq!(t1.token_type, TokenType::Import);

    let t2 = lexer.get_next_token(false, None);
    assert_eq!(t2.token_type, TokenType::As);
}

#[test]
fn test_lexer_optional() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("u8?");

    let t1 = lexer.get_next_token(false, None);
    assert_eq!(t1.token_type, TokenType::Primitive);

    let t2 = lexer.get_next_token(false, None);
    assert_eq!(t2.token_type, TokenType::Optional);
}

#[test]
fn test_lexer_array() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("u8[10] u8[] u8[0..20]");

    // u8
    assert_eq!(
        lexer.get_next_token(false, None).token_type,
        TokenType::Primitive
    );
    // [10]
    let arr = lexer.get_next_token(false, None);
    assert_eq!(arr.token_type, TokenType::Array);

    // u8
    assert_eq!(
        lexer.get_next_token(false, None).token_type,
        TokenType::Primitive
    );
    // []
    let arr2 = lexer.get_next_token(false, None);
    assert_eq!(arr2.token_type, TokenType::Array);

    // u8
    assert_eq!(
        lexer.get_next_token(false, None).token_type,
        TokenType::Primitive
    );
    // [0..20]
    let arr3 = lexer.get_next_token(false, None);
    assert_eq!(arr3.token_type, TokenType::Array);
}

#[test]
fn test_lexer_range() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("(0..100)");

    let token = lexer.get_next_token(false, None);
    assert_eq!(token.token_type, TokenType::Range);
}

#[test]
fn test_lexer_comments_skipped() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("-- this is a comment\ntype");

    let token = lexer.get_next_token(false, None);
    assert_eq!(token.token_type, TokenType::Keyword);
    assert_eq!(token.value, TokenValue::Str("type".to_string()));
}

#[test]
fn test_lexer_class() {
    let mut lexer = Lexer::new(Some(LexerMode::Parsing));
    lexer.initialize("(Sound)");

    let token = lexer.get_next_token(false, None);
    assert_eq!(token.token_type, TokenType::Class);
}

// ---- Parser Tests ----

#[test]
fn test_parse_simple_type() {
    let source = "type A = u8";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].name, "A");
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Primitive);
}

#[test]
fn test_parse_enum() {
    let source = "enum States = { A, B, C, D }";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].name, "States");
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Enum);

    if let TypeNodeValue::Enum { ref values, .. } = body.declarations[0].value {
        assert_eq!(values, &vec!["A", "B", "C", "D"]);
    } else {
        panic!("Expected Enum variant");
    }
}

#[test]
fn test_parse_set() {
    let source = "set Flags = {F1, F2, F3}";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].name, "Flags");
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Set);

    if let TypeNodeValue::Set { ref values, .. } = body.declarations[0].value {
        assert_eq!(values, &vec!["F1", "F2", "F3"]);
    } else {
        panic!("Expected Set variant");
    }
}

#[test]
fn test_parse_struct() {
    let source = "struct Example { Field: u8, Name: string }";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].name, "Example");
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Struct);

    if let TypeNodeValue::Struct { ref values, .. } = body.declarations[0].value {
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].name, "Field");
        assert_eq!(values[1].name, "Name");
    } else {
        panic!("Expected Struct variant");
    }
}

#[test]
fn test_parse_map() {
    let source = "map MapSimple = {[string]: u8}";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].name, "MapSimple");
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Map);
}

#[test]
fn test_parse_event() {
    let source = r#"
        event TestEvent {
            From: Server,
            Type: Reliable,
            Call: SingleSync,
            Data: u8
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].name, "TestEvent");
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Event);

    if let TypeNodeValue::Event {
        ref from,
        ref event_type,
        ref call,
        ref data,
    } = body.declarations[0].value
    {
        assert_eq!(from, "Server");
        assert_eq!(event_type, "Reliable");
        assert_eq!(call, "SingleSync");
        assert!(data.is_some());
    } else {
        panic!("Expected Event variant");
    }
}

#[test]
fn test_parse_function() {
    let source = r#"
        function RemoteFunction {
            Yield: Coroutine,
            Data: u8,
            Return: u8
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].name, "RemoteFunction");
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Function);

    if let TypeNodeValue::Function {
        ref yield_type,
        ref data,
        ref return_type,
    } = body.declarations[0].value
    {
        assert_eq!(yield_type, "Coroutine");
        assert!(data.is_some());
        assert!(return_type.is_some());
    } else {
        panic!("Expected Function variant");
    }
}

#[test]
fn test_parse_options() {
    let source = r#"
        option Casing = Pascal
        option Typescript = true
        option ClientOutput = "Client.luau"
        option ServerOutput = "Server.luau"
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.options.casing, Some("Pascal".to_string()));
    assert_eq!(body.options.typescript, Some(true));
    assert_eq!(body.options.client_output, Some("Client.luau".to_string()));
    assert_eq!(body.options.server_output, Some("Server.luau".to_string()));
}

#[test]
fn test_parse_optional_type() {
    let source = "type A = u8?";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Optional);
}

#[test]
fn test_parse_array_type() {
    let source = "type A = u8[10]";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Array);
}

#[test]
fn test_parse_reference() {
    let source = r#"
        type Byte = u8
        type MyByte = Byte
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 2);
    assert_eq!(body.declarations[1].name, "MyByte");
    assert_eq!(body.declarations[1].decl_type, DeclarationType::Primitive);
}

#[test]
fn test_parse_export() {
    let source = "export type Byte = u8";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { export, .. } = &body.declarations[0].value {
        assert_eq!(*export, Some(true));
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_scope() {
    let source = r#"
        scope MyScope {
            type A = u8
            event TestEvent {
                From: Server,
                Type: Reliable,
                Call: SingleSync,
                Data: A
            }
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Scope);
}

#[test]
fn test_parse_tag_enum() {
    let source = r#"
        enum Event = "Type" {
            Join {
                Name: string,
                UserId: f64,
            },
            Leave {
                UserId: f64
            }
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::TagEnum);

    if let TypeNodeValue::TagEnum {
        ref tag,
        ref values,
        ..
    } = body.declarations[0].value
    {
        assert_eq!(tag, "Type");
        assert_eq!(values.len(), 2);
    } else {
        panic!("Expected TagEnum variant");
    }
}

#[test]
fn test_parse_tuple() {
    let source = r#"
        event Test {
            From: Server,
            Type: Reliable,
            Call: SingleSync,
            Data: (u8, string, boolean)
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Event { ref data, .. } = body.declarations[0].value {
        let data = data.as_ref().unwrap();
        assert_eq!(data.node_type, DeclarationType::Tuple);
    } else {
        panic!("Expected Event variant");
    }
}

#[test]
fn test_parse_generics() {
    let source = r#"
        struct Generic<A, B> {
            Data: A,
            More: B
        }
        event TestGeneric {
            From: Server,
            Type: Reliable,
            Call: SingleSync,
            Data: Generic<u8, string>
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 2);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Struct);
    // The event data field references Generic with type params resolved
    assert_eq!(body.declarations[1].decl_type, DeclarationType::Event);
    if let TypeNodeValue::Event { ref data, .. } = body.declarations[1].value {
        let d = data.as_ref().unwrap();
        assert_eq!(d.node_type, DeclarationType::Struct);
    } else {
        panic!("Expected Event variant");
    }
}

#[test]
fn test_parse_nested_struct() {
    let source = r#"
        struct Example {
            Nested: struct {
                Value: u8
            }
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Struct { ref values, .. } = body.declarations[0].value {
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "Nested");
        assert_eq!(values[0].node_type, DeclarationType::Struct);
    } else {
        panic!("Expected Struct variant");
    }
}

#[test]
fn test_parse_struct_merge() {
    let source = r#"
        struct A { X: u8 }
        struct B { Y: string }
        struct C { ..A, ..B, Z: boolean }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 3);
    if let TypeNodeValue::Struct { ref values, .. } = body.declarations[2].value {
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].name, "X");
        assert_eq!(values[1].name, "Y");
        assert_eq!(values[2].name, "Z");
    } else {
        panic!("Expected Struct variant");
    }
}

#[test]
fn test_parse_polling_event() {
    let source = r#"
        event PollEvent {
            From: Client,
            Type: Reliable,
            Call: SingleSync,
            Poll: true
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Event { ref call, .. } = body.declarations[0].value {
        assert_eq!(call, "Polling");
    } else {
        panic!("Expected Event variant");
    }
}

#[test]
fn test_parse_empty_event() {
    let source = r#"
        event EmptyEvent {
            From: Server,
            Type: Reliable,
            Call: SingleSync
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Event { ref data, .. } = body.declarations[0].value {
        assert!(data.is_none());
    } else {
        panic!("Expected Event variant");
    }
}

#[test]
fn test_parse_empty_function() {
    let source = r#"
        function EmptyFunction {
            Yield: Coroutine
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Function {
        ref data,
        ref return_type,
        ..
    } = body.declarations[0].value
    {
        assert!(data.is_none());
        assert!(return_type.is_none());
    } else {
        panic!("Expected Function variant");
    }
}

#[test]
fn test_parse_multi_dimensional_array() {
    let source = "type A = u8[][][]";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Array);

    // Should be Array of Array of Array of u8
    if let TypeNodeValue::Array { ref of, .. } = body.declarations[0].value {
        assert_eq!(of.node_type, DeclarationType::Array);
        if let TypeNodeValue::Array { ref of, .. } = of.value {
            assert_eq!(of.node_type, DeclarationType::Array);
            if let TypeNodeValue::Array { ref of, .. } = of.value {
                assert_eq!(of.node_type, DeclarationType::Primitive);
            }
        }
    } else {
        panic!("Expected Array variant");
    }
}

#[test]
fn test_parse_range() {
    let source = "type A = f32(0..100)";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref range, .. } = body.declarations[0].value {
        let r = range.as_ref().unwrap();
        assert_eq!(r.min, 0.0);
        assert_eq!(r.max, 100.0);
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_negative_range() {
    let source = "type A = i8(-5..10)";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref range, .. } = body.declarations[0].value {
        let r = range.as_ref().unwrap();
        assert_eq!(r.min, -5.0);
        assert_eq!(r.max, 10.0);
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_components() {
    let source = "type A = vector<i16>";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref components, .. } = body.declarations[0].value {
        let comps = components.as_ref().unwrap();
        assert_eq!(comps, &vec!["i16"]);
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_cframe_components() {
    let source = "type A = CFrame<f32, f16>";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref components, .. } = body.declarations[0].value {
        let comps = components.as_ref().unwrap();
        assert_eq!(comps, &vec!["f32", "f16"]);
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_instance_class() {
    let source = "type A = Instance(Sound)";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref class, .. } = body.declarations[0].value {
        assert_eq!(class.as_ref().unwrap(), "Sound");
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_map_reference() {
    let source = r#"
        enum States = { A, B }
        map MapRef = {[States]: States}
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 2);
    assert_eq!(body.declarations[1].decl_type, DeclarationType::Map);
}

#[test]
fn test_parse_cross_scope_reference() {
    let source = r#"
        scope AnotherScope {
            scope Inner {
                type ExampleType = u8
            }
        }
        scope MyScope {
            event TestEvent {
                From: Server,
                Type: Reliable,
                Call: SingleSync,
                Data: AnotherScope.Inner.ExampleType
            }
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 2);
}

#[test]
fn test_parse_struct_with_string_keys() {
    let source = r#"
        struct TestStruct {
            ["z z"]: u8,
            ["1 1"]: u8
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Struct { ref values, .. } = body.declarations[0].value {
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].name, "z z");
        assert_eq!(values[1].name, "1 1");
    } else {
        panic!("Expected Struct variant");
    }
}

#[test]
fn test_parse_generic_map() {
    let source = r#"
        map GenericMap<K, V> = {[K]: V}
        map Concrete = GenericMap<string, u8>
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 2);
    assert_eq!(body.declarations[1].decl_type, DeclarationType::Map);
}

#[test]
fn test_parse_decimal_range() {
    let source = "type A = f32(-5.5..10.5)";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref range, .. } = body.declarations[0].value {
        let r = range.as_ref().unwrap();
        assert_eq!(r.min, -5.5);
        assert_eq!(r.max, 10.5);
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_unbound_upper_range() {
    let source = "type A = f32(-0.0..)";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref range, .. } = body.declarations[0].value {
        let r = range.as_ref().unwrap();
        assert_eq!(r.min, 0.0);
        assert_eq!(r.max, 16777216.0); // f32 bounds max
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_unbound_lower_range() {
    let source = "type A = f32(..-0.0)";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref range, .. } = body.declarations[0].value {
        let r = range.as_ref().unwrap();
        assert_eq!(r.min, -16777216.0); // f32 bounds min
        assert_eq!(r.max, 0.0);
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_buffer_type() {
    let source = "type A = buffer";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Primitive);
}

#[test]
fn test_parse_exact_buffer() {
    let source = "type A = buffer(9)";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Primitive { ref range, .. } = body.declarations[0].value {
        let r = range.as_ref().unwrap();
        assert_eq!(r.min, 9.0);
        assert_eq!(r.max, 9.0);
    } else {
        panic!("Expected Primitive variant");
    }
}

#[test]
fn test_parse_many_event_types() {
    let source = r#"
        event ManySync {
            From: Server,
            Type: Reliable,
            Call: ManySync,
            Data: u8
        }
        event ManyAsync {
            From: Server,
            Type: Reliable,
            Call: ManyAsync,
            Data: u8
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 2);
    if let TypeNodeValue::Event { ref call, .. } = body.declarations[0].value {
        assert_eq!(call, "ManySync");
    }
    if let TypeNodeValue::Event { ref call, .. } = body.declarations[1].value {
        assert_eq!(call, "ManyAsync");
    }
}

#[test]
fn test_parse_set_with_string_values() {
    let source = r#"set TestFlags = { "a a" }"#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Set { ref values, .. } = body.declarations[0].value {
        assert_eq!(values, &vec!["a a"]);
    }
}

#[test]
fn test_parse_enum_with_string_values() {
    let source = r#"enum TestEnums = { "b b" }"#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Enum { ref values, .. } = body.declarations[0].value {
        assert_eq!(values, &vec!["b b"]);
    }
}

#[test]
fn test_parse_tag_enum_with_string_keys() {
    let source = r#"
        enum TestTaggedEnums = "Type" {
            ["c c"] {
                Test: u8
            }
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::TagEnum { ref values, .. } = body.declarations[0].value {
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "c c");
    }
}

#[test]
fn test_parse_complex_source() {
    // A complex source that tests many features
    let source = r#"
        option Casing = Pascal
        option Typescript = true

        type Byte = u8
        export type Color = Color3
        
        enum States = { A, B, C, D }
        set Flags8 = {F1, F2, F3, F4, F5, F6, F7, F8}
        
        map MapSimple = {[string]: u8}

        struct Generic<A, B, C> {
            Data: A,
            Array: A[],
            Optional: C?,
            Nested: struct {
                Value: B
            }
        }

        struct Example {
            Field: u8?,
            Enum: States,
            Nested: struct {
                Guh: u8,
                Array: u8[10]
            }
        }

        event ReliableServer {
            From: Server,
            Type: Reliable,
            Call: SingleSync,
            Data: u8
        }

        event Generic {
            From: Server,
            Type: Reliable,
            Data: Generic<u8, u8, u8>,
            Call: SingleSync
        }

        function RemoteFunction {
            Yield: Coroutine,
            Data: u8,
            Return: u8
        }

        scope AnotherScope {
            event InScopeEvent {
                From: Server,
                Type: Reliable,
                Call: SingleSync,
                Data: u8
            }
        }
    "#;

    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    // Verify options
    assert_eq!(body.options.casing, Some("Pascal".to_string()));
    assert_eq!(body.options.typescript, Some(true));

    // Count declarations (type, export type, enum, set, map, generic struct, example struct,
    // reliable event, generic event, function, scope)
    assert!(body.declarations.len() >= 10);
}

#[test]
fn test_parse_unit_function() {
    let source = r#"
        function UnitFunction {
            Yield: Coroutine,
            Data: (),
            Return: ()
        }
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    if let TypeNodeValue::Function {
        ref data,
        ref return_type,
        ..
    } = body.declarations[0].value
    {
        // Unit tuple - empty tuple
        assert!(data.is_some());
        assert!(return_type.is_some());
        let d = data.as_ref().unwrap();
        assert_eq!(d.node_type, DeclarationType::Tuple);
        if let TypeNodeValue::Tuple { ref values, .. } = d.value {
            assert_eq!(values.len(), 0);
        }
    } else {
        panic!("Expected Function variant");
    }
}

#[test]
fn test_parse_map_nested() {
    let source = r#"
        map Inner = {[string]: u8}
        map Outer = {[Inner]: Inner}
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 2);
    assert_eq!(body.declarations[1].decl_type, DeclarationType::Map);
}

#[test]
fn test_parse_optional_array() {
    let source = "type A = u8[10]?";
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Optional);
    if let TypeNodeValue::Optional { ref of, .. } = body.declarations[0].value {
        assert_eq!(of.node_type, DeclarationType::Array);
    }
}

#[test]
fn test_parse_array_of_structs() {
    let source = r#"
        struct ArrayStruct {

        }[0..20]
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 1);
    assert_eq!(body.declarations[0].decl_type, DeclarationType::Array);
}

#[test]
fn test_parse_map_optional() {
    let source = r#"
        enum S = { A }
        map M = {[S]: S}?
    "#;
    let mut parser = Parser::new(None, None);
    let body = parser.parse(source, None);

    assert_eq!(body.declarations.len(), 2);
    assert_eq!(body.declarations[1].decl_type, DeclarationType::Optional);
}
