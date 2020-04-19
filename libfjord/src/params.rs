//! Data structures for representing function parameters, both at definition and call sites of
//! functions.

pub mod call;
pub mod def;

#[derive(Debug)]
pub(crate) struct CompleteParam {
    pub name: crate::IdentName,
    pub val: crate::Expr,
}

/// The errors that can occur when matching up function definition parameters with function call
/// parameters.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// when a named function parameter is specified, but it actually does not exist
    #[error("could not find function parameter")]
    FuncParamNotFound,
    /// when too many function parameters are specified
    #[error("too many function parameters were provided")]
    TooManyFuncParams,
    /// when not enough function parameters are specified
    #[error("not enough function parameters were provided")]
    NotEnoughFuncParams,
}

pub(crate) fn eval(
    call_params: Vec<call::Param>,
    mut def_params: Vec<def::Param>,
) -> Result<Vec<CompleteParam>, Error> {
    use std::cmp::Ordering;

    let mut complete_params = Vec::new();

    let (named_params, positional_params): (Vec<_>, Vec<_>) =
        call_params.into_iter().partition(|p| p.is_named());

    // Loop through all named function parameters. If the parameter actually exists, add it to the
    // vector of completed parameters and remove it from the list of function definition params (as
    // we don’t need it anymore). If it doesn’t exist, then return an error.
    for named_param in named_params {
        // At this point we know that the parametere is named, so we know that name is Some.
        let param_name = named_param.name.unwrap();

        if let Some(def_param_idx) = def_params.iter().position(|p| p.name == param_name) {
            complete_params.push(CompleteParam {
                name: param_name,
                val: named_param.val,
            });
            def_params.remove(def_param_idx);
        } else {
            return Err(Error::FuncParamNotFound);
        }
    }

    let def_params_len = def_params.len();
    let positional_params_len = positional_params.len();

    let ord = positional_params_len.cmp(&def_params_len);
    match ord {
        // In this case there are the same number or less positional paramters than remaining
        // definition arguments.
        Ordering::Less | Ordering::Equal => {
            // Match up all the call parameters with as many definition parameters as possible.
            //
            // zip stops yielding elements when one of the iterators returns None, thereby limiting
            // the length of the iterator to the shortest of the two inputs.
            for (call_param, def_param) in positional_params.into_iter().zip(&def_params) {
                complete_params.push(CompleteParam {
                    name: def_param.name.clone(),
                    val: call_param.val,
                });
            }

            // If there are less positional parameters than definition parameters, then this means
            // that the caller of the function has omitted some parameters.

            if ord == Ordering::Less {
                // Remove all the function definition parameters we just used as they aren’t needed
                // anymore. We only do this in this branch because it doesn’t affect anything if
                // all parameters have a value.
                (0..positional_params_len).for_each(|_| {
                    def_params.remove(0);
                });

                // The first thing we can do to try to fill in some of the missing parameters is to
                // use all the remaining definition parameters’ default values (if they have any).

                let default_params = def_params.iter().filter(|p| p.has_default());

                for default_param in default_params {
                    complete_params.push(CompleteParam {
                        name: default_param.name.clone(),
                        // At this point we know the parameter has a default value, so we can
                        // safely unwrap here.
                        val: default_param.val.as_ref().unwrap().clone(),
                    });
                }

                // We don’t need these any more.
                def_params.retain(|p| !p.has_default());

                // If everything has gone well, there should be no definition parameters left.
                // However, if the caller has not specified enough parameters, there will be some
                // left over.
                if !def_params.is_empty() {
                    return Err(Error::NotEnoughFuncParams);
                }
            }
        }
        // In this case we have more input arguments than are defined on the function.
        Ordering::Greater => {
            return Err(Error::TooManyFuncParams);
        }
    }

    Ok(complete_params)
}
