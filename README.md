# lytelog

lytelog is a Rust logging library containing the macros necessary to have lightweight logging. 
It is particularly aimed toward use in command line tools which have enumerated tasks to complete and
want to notify the user of progress in an elegant way.

## Structure

lytelog is structured around tasks. 
A task may be begun at any time using the `start!` macro along with a format string and 
arguments that are immediately printed next to the spinner.

To initiate a subtask, simply use the `start!` macro once again, 
and it will create a subtask directly underneath the existing task.

To complete a task, use either the `pass!`, `warn!`, or `fail!` 
macro to indicate to the user the final state of the task.

## Considerations

Actively spinning the spinner requires an additional thread. 
In the future, there will be a feature flag that toggles spawning an async task instead, 
but this is not yet implemented. This extra thread spends most of its time sleeping, 
as it sleeps for 80ms between updating the spinner. 
It should not have much of an impact on the performance of your program. 
However, it is still important to be aware that running a task does require this extra thread, 
even though it does not require many resources. When a task is not running, 
there is no concurrent spinner thread.
