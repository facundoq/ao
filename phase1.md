1. Make sure all commands are implemented with a Command design pattern, where a struct has fields that implement the required functionality (execute, print..) 
2. Add comprehensive tests using the "print" function that each command must implement, testing that the command is ok for a given platform/feature set.
3. Generalize the solution so that other linux distributions can be used, ie, apt can be used in debian, ubuntu, etc, and similarly with systemd based systemctl, etc. Make sure all tests pass. Add new tests as necessary.
4. Review the feature matrix support For each item that is not supported:
  3. Implement the missing feature. Make sure all tests pass. Add new tests as necessary.
5. Add support for arch linux based distros and fedora based distros. Make sure all tests pass. Add new tests as necessary.
