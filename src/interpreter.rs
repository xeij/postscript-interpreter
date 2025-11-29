//! PostScript Interpreter Execution Engine
//!
//! This module implements the core execution logic for the PostScript interpreter.
//! It uses a stack-based execution model where values are popped from the execution
//! stack and processed according to their type.

use crate::types::{Context, PostScriptValue};

/// The interpreter executes PostScriptValue objects using a Context.
///
/// # Execution Model
///
/// The interpreter operates on three stacks (all stored in Context):
/// - **Execution stack**: Values waiting to be executed (LIFO queue)
/// - **Operand stack**: Values used for computation (arguments and results)
/// - **Dictionary stack**: Hierarchical namespace for variable lookup
///
/// # Execution Flow
///
/// 1. Values are pushed onto the execution stack (in reverse order)
/// 2. The interpreter pops each value and executes it:
///    - Literals (Int, Real, String, etc.) → pushed to operand stack
///    - Names → looked up in dictionary stack and executed
///    - Blocks → pushed to operand stack (or converted to Closures in lexical mode)
///    - NativeFn → called with mutable Context reference
///    - Loops → managed on execution stack with state preservation
/// 3. Repeat until execution stack is empty
pub struct Interpreter {
    context: Context,
}

impl Interpreter {
    /// Creates a new interpreter with the given context.
    pub fn new(context: Context) -> Self {
        Interpreter { context }
    }

    /// Executes a sequence of PostScriptValue objects.
    ///
    /// Values are pushed onto the execution stack in reverse order so that
    /// the first value in the input vector is executed first.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Execute "3 4 add" (pushes 7 to operand stack)
    /// let values = vec![
    ///     PostScriptValue::Int(3),
    ///     PostScriptValue::Int(4),
    ///     PostScriptValue::Name("add".to_string()),
    /// ];
    /// interpreter.execute(values)?;
    /// ```
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

    /// Executes a single PostScriptValue.
    ///
    /// This is the heart of the interpreter. It handles each value type differently:
    ///
    /// - **Name**: Look up in dictionary stack and execute the result
    /// - **Block**: Push to operand stack (or convert to Closure in lexical mode)
    /// - **NativeFn**: Call the function with mutable Context
    /// - **ForLoop/RepeatLoop**: Manage loop iteration on execution stack
    /// - **Closure**: Execute with captured environment
    /// - **RestoreEnv**: Restore dictionary stack after closure execution
    /// - **Literals**: Push directly to operand stack
    fn execute_one(&mut self, value: PostScriptValue) -> Result<(), String> {
        match value {
            PostScriptValue::Name(ref name) => {
                // Look up the name in the dictionary stack
                if let Some(val) = self.context.lookup(name) {
                    match val {
                        // Native function: call it immediately
                        PostScriptValue::NativeFn(f) => f(&mut self.context)?,
                        
                        // Block: push contents to execution stack for execution
                        PostScriptValue::Block(block) => {
                            for item in block.iter().rev() {
                                self.context.execution_stack.push(item.clone());
                            }
                        }
                        
                        // Closure: execute with captured environment
                        PostScriptValue::Closure { body, env } => {
                            // Save current environment for restoration
                            self.context.execution_stack.push(PostScriptValue::RestoreEnv(self.context.dict_stack.clone()));
                            // Switch to closure's captured environment
                            self.context.dict_stack = env;
                            // Push closure body to execution stack
                            for item in body.iter().rev() {
                                self.context.execution_stack.push(item.clone());
                            }
                        }
                        
                        // Other values: push to operand stack
                        _ => self.context.push(val),
                    }
                } else {
                    return Err(format!("Undefined name: {}", name));
                }
            }
            PostScriptValue::Block(block) => {
                // Literal block (procedure)
                if self.context.lexical_scoping {
                    // In lexical scoping mode, capture current environment as a closure
                    self.context.push(PostScriptValue::Closure {
                        body: block,
                        env: self.context.dict_stack.clone(),
                    });
                } else {
                    // In dynamic scoping mode, just push the block
                    self.context.push(PostScriptValue::Block(block));
                }
            }
            PostScriptValue::ForLoop { current, step, limit, proc } => {
                // For-loop execution: "initial step limit proc for"
                // Continues while: (step > 0 && current <= limit) || (step < 0 && current >= limit)
                let continue_loop = if step > 0.0 { current <= limit } else { current >= limit };
                
                if continue_loop {
                    // Push next iteration state back onto execution stack
                    self.context.execution_stack.push(PostScriptValue::ForLoop {
                        current: current + step,
                        step,
                        limit,
                        proc: proc.clone(),
                    });
                    
                    // Push current loop index onto operand stack (available to procedure)
                    self.context.push(PostScriptValue::Real(current));
                    
                    // Execute the procedure with the current index on the stack
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
                // Repeat-loop execution: "n proc repeat"
                // Executes proc n times
                if count > 0 {
                    // Push next iteration state back onto execution stack
                    self.context.execution_stack.push(PostScriptValue::RepeatLoop {
                        count: count - 1,
                        proc: proc.clone(),
                    });
                    
                    // Execute the procedure
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
                // Restore dictionary stack after closure execution
                self.context.dict_stack = env;
            }
            
            // All other values (literals) are pushed to the operand stack
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
