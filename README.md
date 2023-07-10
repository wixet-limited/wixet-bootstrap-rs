# wixet-bootstrap

First of all, this library only makes sense if you use `tokio` and your application runs in the asynchronous way.

Some code is used everytime. This library is the case for Wixet. If there is something common in all applications, this is
a boot process. For simple programs or a "hello world" it is just simple but as long as your application grows you always
implement it somehow.

This library provides a simple but useful minimal boot process:
* Configure logger (format and option to write a file) using [fern](https://github.com/daboross/fern)
* Friendly exit and interrupt handler (ctr+c...) using [signal-hook](https://github.com/vorner/signal-hook)

# How to use it

Take this simple example
```rust
use wixet_bootstrap::init;
use log::info;

#[tokio::main]
async fn main() {
    info!("This log line will be ignored because the logger is not configured yet");
    let (closer, exit) = init(Some("output.log"), None, None).await?; //If you provide None, it simple will not write a log file (just output)
    info!("Hello to my application!")

    // Do may awesome stuff spawing tokio tasks

    // I use select here because it is common to listen for multiple signals, but you can just await the `exit` if not
    tokio::select!{
        _ = exit.recv_async() => {
            info!("Shutdown process started");
            // Do your friendly stop process here
            // This code is run when ctrl+c or any other kill interrupt is received
        }
    };

    // A friendly shutdown by deinitializing all "init" stuff.
    closer.stop().await?;
    info!("Bye");

}
```

As you can see, it is very simple and easy to use but it saves many lines of code. And the most important thing, if we add a new feature/improvement it will apply to all the projects.

I want to keep this library as simple and generic as possible but if you find something interesting to add I'll be glad to hear it!

# Extra config

Default log level
-----------------
  
  If you dont provide any log level, info will be used. To set your own preference do it like this: 
  ```
  let (closer, exit) = init(Some("output.log"), Some(log::LevelFilter::Warn), None).await?;
  ```

Log level for other modules
---------------------------
  In case you want to set a default log level but change only the level for some modules, you can provide them in a hashmap:
  ```
  
  let (closer, exit) = init(Some("output.log"), None, None).await?;
  ```