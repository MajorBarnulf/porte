#[test]
fn ast() {
    use crate::execution_tree;
    use crate::syntax_tree;

    let text = r#"
	a: 3;
	{
		a <- 4;
		a: 5
	}
	"#;

    let tree = syntax_tree::parser::ParserWrapper::new()
        .parse(text)
        .unwrap();
    dbg!(&tree);

    let executable = execution_tree::parser::Parser::parse(tree, |_| ());
    dbg!(&executable);
}
