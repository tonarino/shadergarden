use std::rc::Rc;

use glium::backend::Context;
use lexpr::Value;

use crate::{
    graph::{
        NodeId,
        ShaderGraph,
    },
    reload::ShaderDir,
};

mod env;
mod load;
mod val;

pub use env::{
    Env,
    External,
};
pub use load::load_shaders;
pub use val::Val;

/// Takes a source string of lisp that represents the shader
/// graph to be constructed, And constructs that shader
/// graph within a certain context, Provided a map of all
/// shaders avaliable for the construction of the graph. You
/// can use [`load_shaders`] to load a directory into a map.
pub fn graph_from_sexp(
    context: &Rc<Context>,
    shader_dir: ShaderDir,
    external: External,
) -> Result<ShaderGraph, String> {
    let mut graph = ShaderGraph::new(context);
    let mut env = Env::new(shader_dir.shaders, external);

    // little hack to get a list of expressions
    let sexp = lexpr::from_str(&format!("({})", shader_dir.lisp))
        .map_err(|e| format!("{}", e))?;

    begin(&mut graph, &mut env, &sexp)?;

    Ok(graph)
}

fn into_iter(sexp: &Value) -> Result<lexpr::cons::ListIter<'_>, String> {
    sexp.list_iter()
        .ok_or_else(|| "Expected a form".to_string())
}

fn next_item<'a>(
    iter: &mut lexpr::cons::ListIter<'a>,
) -> Result<&'a Value, String> {
    iter.next()
        .ok_or_else(|| "Expected a non-empty form".to_string())
}

fn next_symbol<'a>(
    iter: &mut lexpr::cons::ListIter<'a>,
) -> Result<&'a str, String> {
    return next_item(iter)?
        .as_symbol()
        .ok_or_else(|| "Expected a symbol".to_string());
}

fn iter_finish(iter: lexpr::cons::ListIter<'_>) -> Result<(), String> {
    if !iter.is_empty() {
        Err("Unexpected extra args while parsing form".to_string())
    } else {
        Ok(())
    }
}

fn begin(
    mut graph: &mut ShaderGraph,
    env: &mut Env,
    sexp: &Value,
) -> Result<(), String> {
    for declaration in into_iter(sexp)? {
        declare(&mut graph, env, declaration)?;
    }

    Ok(())
}

fn declare(
    graph: &mut ShaderGraph,
    env: &mut Env,
    sexp: &Value,
) -> Result<(), String> {
    let mut iter = into_iter(sexp)?;
    let keyword = next_symbol(&mut iter)?;

    match keyword {
        "input" => {
            let var = next_symbol(&mut iter)?;
            let input = graph.add_input();
            env.set(var.to_string(), Val::Node(input));
        },
        "output" => {
            let id = next_symbol(&mut iter)?;
            graph.mark_output(env.get(id)?.to_node()?);
        },
        "define" => {
            // get the form defining the signature
            let list = next_item(&mut iter)?;
            let mut signature = into_iter(list)?;
            let name = next_symbol(&mut signature)?;

            // extract all the arguments
            let mut args = vec![];
            for arg in signature {
                args.push(
                    arg.as_symbol()
                        .ok_or_else(|| {
                            "Expected symbol in signature".to_string()
                        })?
                        .to_string(),
                );
            }

            let forms: Vec<Value> = iter.map(|f| f.to_owned()).collect();
            if forms.is_empty() {
                return Err(format!(
                    "Definition `{}` must have at least one expression in body",
                    name
                ));
            }
            env.set_fn(name.to_string(), (args, forms));
            return Ok(());
        },
        "let" => {
            let var = next_symbol(&mut iter)?;
            let val = expr(graph, env, next_item(&mut iter)?)?;
            env.set(var.to_string(), val);
        },
        "repeat" => {
            let times = expr(graph, env, next_item(&mut iter)?)?.to_nat()?;
            let forms: Vec<Value> = iter.map(|f| f.to_owned()).collect();
            for _ in 0..times {
                for form in forms.iter() {
                    declare(graph, env, form)?;
                }
            }
            return Ok(());
        },
        other => {
            return Err(format!(
                "Expected a statement keyword, found `{}`",
                other
            ))
        },
    }

    iter_finish(iter)
}

fn expr(
    graph: &mut ShaderGraph,
    env: &mut Env,
    value: &Value,
) -> Result<Val, String> {
    let val = match value {
        Value::Number(n) => {
            if let Some(float) = n.as_f64() {
                Val::Number(float)
            } else {
                return Err("unexpected number type".to_string());
            }
        },

        Value::Bool(b) => Val::Bool(*b),
        Value::String(_) => Val::String(value.as_str().unwrap().to_string()),
        Value::Symbol(_) => env.get(value.as_symbol().unwrap())?.clone(),

        x if x.is_list() => node(graph, env, x)?,

        other => return Err(format!("unexpected value `{}`", other)),
    };

    Ok(val)
}

fn shader(
    graph: &mut ShaderGraph,
    env: &mut Env,
    mut iter: lexpr::cons::ListIter<'_>,
) -> Result<(String, u32, u32, Vec<NodeId>), String> {
    let name = expr(graph, env, next_item(&mut iter)?)?.to_string()?;
    let width = expr(graph, env, next_item(&mut iter)?)?.to_nat()?;
    let height = expr(graph, env, next_item(&mut iter)?)?.to_nat()?;
    let mut inputs = vec![];
    for remaining in iter {
        let node_id = expr(graph, env, remaining)?.to_node()?;
        inputs.push(node_id);
    }
    Ok((name, width as u32, height as u32, inputs))
}

fn external(
    graph: &mut ShaderGraph,
    env: &mut Env,
    mut iter: lexpr::cons::ListIter<'_>,
) -> Result<(String, Vec<NodeId>), String> {
    let name = expr(graph, env, next_item(&mut iter)?)?.to_string()?;
    let mut inputs = vec![];
    for remaining in iter {
        let node_id = expr(graph, env, remaining)?.to_node()?;
        inputs.push(node_id);
    }
    Ok((name, inputs))
}

fn node(
    graph: &mut ShaderGraph,
    env: &mut Env,
    sexp: &Value,
) -> Result<Val, String> {
    let mut iter = into_iter(sexp)?;
    let function = next_symbol(&mut iter)?;

    match function {
        "shader" => {
            let (name, width, height, inputs) = shader(graph, env, iter)?;
            let node_id = graph.add_shader(
                env.shader(&name)?,
                inputs,
                width as u32,
                height as u32,
            )?;
            Ok(Val::Node(node_id))
        },
        "shader-inline" => {
            let (src, width, height, inputs) = shader(graph, env, iter)?;
            let node_id = graph.add_shader(
                &src,
                inputs,
                width as u32,
                height as u32,
            )?;
            Ok(Val::Node(node_id))
        },
        "shader-param" => {
            // get the shader we'll be running the transformations
            // against
            let decl = into_iter(next_item(&mut iter)?)?;
            let (name, width, height, inputs) = shader(graph, env, decl)?;
            let mut source = env.shader(&name)?.to_string();

            // parse the substitutions to be applied
            // let mut subst = vec![];
            for form in iter {
                source = subst(graph, env, form, source)?;
            }

            let node_id = graph.add_shader(
                &source,
                inputs,
                width as u32,
                height as u32,
            )?;
            Ok(Val::Node(node_id))
        },
        "shader-rec" => {
            let (name, width, height, inputs) = shader(graph, env, iter)?;
            let node_id = graph.add_rec_shader(
                env.shader(&name)?,
                inputs,
                width as u32,
                height as u32,
            )?;
            Ok(Val::Node(node_id))
        },
        "shader-rec-inline" => {
            let (src, width, height, inputs) = shader(graph, env, iter)?;
            let node_id = graph.add_rec_shader(
                &src,
                inputs,
                width as u32,
                height as u32,
            )?;
            Ok(Val::Node(node_id))
        },
        "extern" => {
            let (name, inputs) = external(graph, env, iter)?;
            let adder = env.external(&name)?;
            let node_id = adder(graph, &inputs).map_err(|e| {
                format!("While adding external function `{}`: {}", name, e)
            })?;
            Ok(Val::Node(node_id))
        },
        user_defined => {
            // evaluate the arguments (pass by value)
            let mut args = vec![];
            for arg in iter {
                args.push(expr(graph, env, arg)?);
            }

            if let Some(val) = builtin(user_defined, &args) {
                return val;
            }

            // get the function
            let (params, body) = env.get_fn(user_defined)?.clone();

            // check things match up before calling
            if params.len() != args.len() {
                return Err(format!(
                    "function `{}` expected {} args, but found {}",
                    user_defined,
                    params.len(),
                    args.len(),
                ));
            }

            // evaluate in new scope, declare arguments
            // recursive definitions will bap the stack, so please don't
            // write them
            env.enter_scope();
            for (name, val) in params.iter().zip(args.iter()) {
                env.set(name.to_string(), val.to_owned())
            }

            // unwrap is ok because length is checked when adding
            // definiton
            let last = body.last().unwrap();
            let declarations = &body[..body.len() - 1];
            for declaration in declarations {
                declare(graph, env, declaration)?;
            }

            // TODO: multiple returns how?
            // last value must be an expression, return it
            let ret = expr(graph, env, last)?.to_node()?;
            env.exit_scope();
            Ok(Val::Node(ret))
        },
    }
}

fn builtin(name: &str, args: &[Val]) -> Option<Result<Val, String>> {
    let result: fn(Vec<f64>) -> Val = match name {
        "+" => |n| Val::Number(n.into_iter().sum()),
        "-" => |n| {
            let iter = n.into_iter();
            Val::Number(iter.reduce(|a, b| a - b).unwrap_or(0.0))
        },
        "*" => |n| Val::Number(n.into_iter().product()),
        "/" => |n| {
            let iter = n.into_iter();
            Val::Number(iter.reduce(|a, b| a / b).unwrap_or(1.0))
        },
        _ => {
            return None;
        },
    };

    let numbers = args
        .iter()
        .map(|v| v.to_float())
        .collect::<Result<Vec<f64>, _>>();

    let numbers = match numbers {
        Err(e) => {
            return Some(Err(e));
        },
        Ok(ok) => ok,
    };

    Some(Ok(result(numbers)))
}

fn subst(
    graph: &mut ShaderGraph,
    env: &mut Env,
    form: &Value,
    source: String,
) -> Result<String, String> {
    let mut subst_iter = into_iter(form)?;
    let op = next_symbol(&mut subst_iter)?;

    let (name, subst) = match op {
        "define" => {
            let name =
                expr(graph, env, next_item(&mut subst_iter)?)?.to_string()?;
            let val =
                expr(graph, env, next_item(&mut subst_iter)?)?.to_string()?;
            let subst = format!("#define {} {}", name, val);
            (name, subst)
        },
        "ifdef" => {
            let name =
                expr(graph, env, next_item(&mut subst_iter)?)?.to_string()?;
            let should_define =
                expr(graph, env, next_item(&mut subst_iter)?)?.to_bool()?;
            if should_define {
                (name.clone(), format!("#define {} 1", name))
            } else {
                (name, String::new())
            }
        },
        other => {
            return Err(format!("Invalid param substitution type `{}`", other));
        },
    };

    iter_finish(subst_iter)?;
    return Ok(source.replace(&format!("<{}>", name), &subst));
}
