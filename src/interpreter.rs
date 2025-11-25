use crate::types::{Context, PostScriptValue};

pub struct Interpreter {
    context: Context,
}

impl Interpreter {
    pub fn new(context: Context) -> Self {
        Interpreter { context }
    }

    pub fn execute(&mut self, values: Vec<PostScriptValue>) -> Result<(), String> {
        // Push values to execution stack in reverse order so the first item is at the top
        for value in values.into_iter().rev() {
            self.context.execution_stack.push(value);
        }

        while let Some(value) = self.context.execution_stack.pop() {
            self.execute_one(value)?;
        }
        Ok(())
    }

    fn execute_one(&mut self, value: PostScriptValue) -> Result<(), String> {
        match value {
            PostScriptValue::Name(ref name) => {
                // Look up name
                if let Some(val) = self.context.lookup(name) {
                    match val {
                        PostScriptValue::NativeFn(f) => f(&mut self.context)?,
                        PostScriptValue::Block(block) => {
                            // Execute procedure: push contents to execution stack
                            for item in block.iter().rev() {
                                self.context.execution_stack.push(item.clone());
                            }
                        }
                        PostScriptValue::Closure { body, env } => {
                            // Execute closure
                            // Save current env
                            self.context.execution_stack.push(PostScriptValue::RestoreEnv(self.context.dict_stack.clone()));
                            // Switch env
                            self.context.dict_stack = env;
                            // Push body
                            for item in body.iter().rev() {
                                self.context.execution_stack.push(item.clone());
                            }
                        }
                        _ => self.context.push(val),
                    }
                } else {
                    return Err(format!("Undefined name: {}", name));
                }
            }
            PostScriptValue::Block(block) => {
                // Literal block. If lexical scoping, convert to Closure.
                if self.context.lexical_scoping {
                    self.context.push(PostScriptValue::Closure {
                        body: block,
                        env: self.context.dict_stack.clone(),
                    });
                } else {
                    self.context.push(PostScriptValue::Block(block));
                }
            }
            PostScriptValue::ForLoop { current, step, limit, proc } => {
                let continue_loop = if step > 0.0 { current <= limit } else { current >= limit };
                if continue_loop {
                    // Push next iteration
                    self.context.execution_stack.push(PostScriptValue::ForLoop {
                        current: current + step,
                        step,
                        limit,
                        proc: proc.clone(),
                    });
                    
                    // Push loop index
                    self.context.push(PostScriptValue::Real(current));
                    
                    // Execute proc
                    match *proc {
                        PostScriptValue::Block(ref block) => {
                            for item in block.iter().rev() {
                                self.context.execution_stack.push(item.clone());
                            }
                        }
                        PostScriptValue::Closure { ref body, ref env } => {
                            self.context.execution_stack.push(PostScriptValue::RestoreEnv(self.context.dict_stack.clone()));
                            self.context.dict_stack = env.clone();
                            for item in body.iter().rev() {
                                self.context.execution_stack.push(item.clone());
                            }
                        }
                        _ => self.context.execution_stack.push(*proc),
                    }
                }
            }
            PostScriptValue::RepeatLoop { count, proc } => {
                if count > 0 {
                    // Push next iteration
                    self.context.execution_stack.push(PostScriptValue::RepeatLoop {
                        count: count - 1,
                        proc: proc.clone(),
                    });
                    
                    // Execute proc
                    match *proc {
                        PostScriptValue::Block(ref block) => {
                            for item in block.iter().rev() {
                                self.context.execution_stack.push(item.clone());
                            }
                        }
                        PostScriptValue::Closure { ref body, ref env } => {
                            self.context.execution_stack.push(PostScriptValue::RestoreEnv(self.context.dict_stack.clone()));
                            self.context.dict_stack = env.clone();
                            for item in body.iter().rev() {
                                self.context.execution_stack.push(item.clone());
                            }
                        }
                        _ => self.context.execution_stack.push(*proc),
                    }
                }
            }
            PostScriptValue::RestoreEnv(env) => {
                self.context.dict_stack = env;
            }
            // Literal values are pushed to operand stack
            _ => {
                self.context.push(value);
            }
        }
        Ok(())
    }
    
    pub fn get_context(&self) -> &Context {
        &self.context
    }
    
    pub fn get_context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}
