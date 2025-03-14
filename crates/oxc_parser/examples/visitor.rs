#![expect(clippy::print_stdout)]
use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::{BinaryExpression, Expression, IdentifierName, NumberBase, VariableDeclarator};
use oxc_ast::AstBuilder;
use oxc_ast_visit::VisitMut;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use std::{env, path::Path};
// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example visitor`
// or `cargo watch -x "run -p oxc_parser --example visitor"`

fn main() -> std::io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
    }

    // Get the AST program.
    let mut program = ret.program;

    // Create a PiReplacer with an AstBuilder.
    let mut pi_replacer = PiReplacer { ast_builder: AstBuilder::new(&allocator), pi_value: None };

    // Apply the transformation: any expression that is an identifier "PI"
    // will be replaced with the numeric literal 3.14.
    pi_replacer.visit_program(&mut program);

    // Optionally, you can count again or further process the transformed AST.
    // For example, printing the AST or running additional passes.

    // println!("AST after transformation: {:#?}", program);

    // Generate source code from the transformed AST.
    let printed = CodeGenerator::new()
        .with_options(CodegenOptions { minify: false, ..CodegenOptions::default() })
        .build(&program)
        .code;
    println!("Generated source code:");
    println!("{printed}");

    Ok(())
}


struct PiReplacer<'a> {
    ast_builder: AstBuilder<'a>,
    pi_value: Option<f64>,
}

impl<'a> VisitMut<'a> for PiReplacer<'a> {



    fn visit_variable_declarator(&mut self, it: &mut VariableDeclarator<'a>) {
        let identifier_name = it.id.get_identifier_name().unwrap();
        if identifier_name == "PI" {
            if let Some(Expression::NumericLiteral(literal)) = &it.init {
                self.pi_value = Some(literal.value);
            }
        }
    }


    fn visit_binary_expression(&mut self, it: &mut BinaryExpression<'a>) {
        self.visit_expression(&mut it.left);
        self.visit_expression(&mut it.right);

        if let Expression::Identifier(ident) = &mut it.left {
            if ident.name == "PI" {
                let pi_value = self.pi_value.unwrap();
                it.left = self.ast_builder.expression_numeric_literal(
                    ident.span,
                    pi_value,
                    None,
                    NumberBase::Decimal,
                );
            }
        }
        if let Expression::Identifier(ident) = &mut it.right {
            if ident.name == "PI" {
                println!("Binary right {:?}", ident);
                let pi_value = self.pi_value.unwrap();
                it.right = self.ast_builder.expression_numeric_literal(
                    ident.span,
                    pi_value,
                    None,
                    NumberBase::Decimal,
                );
            }
        }
    }



    /*
    fn visit_variable_declarator(&mut self, it: &mut VariableDeclarator<'a>) {
        let identifier_name = it.id.get_identifier_name().unwrap();
        if identifier_name == "PI" {
            if let Some(Expression::NumericLiteral(_literal)) = &it.init {
                it.init = Some(self.ast_builder.expression_numeric_literal(
                    it.span,
                    3.0,
                    None,
                    NumberBase::Decimal,
                ));
            }
        }
    }

     */





    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::Identifier(ident) if ident.name == "PI" => {
                let pi_value = self.pi_value.unwrap();
                *expr = self.ast_builder.expression_numeric_literal(
                    ident.span,
                    pi_value,
                    None,
                    NumberBase::Decimal,
                );
            }
            Expression::BinaryExpression(binary_expr) => {
                self.visit_binary_expression(binary_expr);
            }
            // ... handle other expression variants as needed
            _ => {
                // For other variants, if needed, manually traverse child nodes here.
            }
        }
    }






}

fn codegen(ret: &ParserReturn<'_>) -> String {
    CodeGenerator::new()
        .with_options(CodegenOptions { ..CodegenOptions::default() })
        .build(&ret.program)
        .code
}
