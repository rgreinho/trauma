---
name: Bug report
about: Create a report to help us improve
labels: 'kind/bug'
---

<!-- Provide a general summary of the issue in the title above. -->

Current Behavior
----------------
<!-- Tell us what is currently happening. -->


Expected Behavior
-----------------
<!--
Tell us how it should work, how it differs from the current implementation.
-->


Possible Solution
-----------------
<!--
Suggest a fix/reason for the bug, or ideas how to implement it.
Delete if not applicable/relevant.
-->


Steps to Reproduce
------------------
<!--
Provide a link to a live example, or an unambiguous set of steps to
reproduce this bug. Include code to reproduce, if relevant.
-->
1.
2.
3.


Context
-------
<!--
How has this issue affected you? What are you trying to accomplish?
Providing context helps us come up with a solution that is most useful
in the real world.
-->


Your Environment
----------------
<!--
Instructions:
  * Run the following script in a terminal (OSX only)
  * Paste the output in the code section at the bottom of this report
    (the output is automatically copied to your clipboard buffer)
  * Adjust the values if needed
  * If you cannot run the script for any reason, simply replace the
    values in the example

COMMIT=$(git log -1 --pretty=format:"%h %s %d")
FIREFOX=$(/Applications/Firefox.app/Contents/MacOS/firefox --version 2>/dev/null||true)
CHROME=$(/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --version 2>/dev/null||true)
SYSTEM=$(system_profiler SPSoftwareDataType|grep macOS | xargs)
OUTPUT="$(cat <<EOF
Last commit:
  ${COMMIT}
Browser(s):
  ${FIREFOX}
  ${CHROME}
${SYSTEM}
EOF
)"
echo "$OUTPUT" | tee >(pbcopy)

-->
```
(replace the example bellow with the script output)
Last commit:
  583bc87 Fix configuration issue
Browser(s):
  Mozilla Firefox 60.0
  Google Chrome 66.0.3359.139
System Version: macOS 10.13.4 (17E202)
```
