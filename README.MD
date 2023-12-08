# Downloads Counter

How to use:

Step 1: Save the urls of the files to be downloaded in URLS.json file as below:
```json
[
    "https://www.rfc-editor.org/rfc/rfc2606.html",
    "https://raw.githubusercontent.com/second-state/llama-utils/main/run-llm.sh"
]
```


Step 2: Push the updated URLS.json file along with your flow function's code to GitHub

Step 3: Go get the webhook url on flows.network

Step 4: Start using the downloads counter by replacing the original download link with the new link assembled like this:
```
https://code.flows.network/webhook/5xAx1Yru2BqZ9PUacb3q?file=FILENAME
```


Step 5: Check the download counts of the files you're tracking at flow function's logs

