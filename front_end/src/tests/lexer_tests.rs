#[test]
fn test_actor_lexing() {
    let source = r#"
        actor Counter {
            reads write count: i64

            fn increment_by(n: i64) -> i64 {
                count += n
            }

            on increment() {
                atomic {
                    count += 1
                }
            }
        }
    "#;

    let lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();

    assert_eq!(tokens[0].token_type, TokenType::Actor);
    assert_eq!(tokens[1].token_type, TokenType::Identifier("Counter".to_string()));
    assert_eq!(tokens[2].token_type, TokenType::LeftBrace);
    // ...additional assertions...
}