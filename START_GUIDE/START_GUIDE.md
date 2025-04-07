# Start Guide (Real-World Mode)
## Account Setup
To use 'real world-mode' you must setup a Square developer account at:\
https://app.squareup.com/signup/en-US?return_to=https%3A%2F%2Fdeveloper.squareup.com%2Fapps&v=developers \
<img src="img_2.png" alt="drawing" width="200"/> \
Setup an application:\
![img_5.png](img_5.png)\
Once you've setup your application - we need to create test accounts. Click the back button:
![img_6.png](img_6.png)
And setup each account you want to test.
![img_7.png](img_7.png)
Return to your application:
![img_8.png](img_8.png)
Go to the OAuth Tab:
![img_9.png](img_9.png)
There you can create tokens for each of your accounts.
## Product Configuration.
On each account you must setup a product- return to the accounts screen:
![img_10.png](img_10.png)
For each, enter the dashboard and create a product:
![img_11.png](img_11.png)
Setup physical goods - ![img_12.png](img_12.png)
You can leave most of this blank:
![img_13.png](img_13.png)
Scroll down to 'variations and stock':
![img_14.png](img_14.png)
And enable tracking:
![img_15.png](img_15.png)
Save the product:
![img_16.png](img_16.png)
For each account you will test with record-mode create two products- a calibration item and a tracked item.\
For each polling platform you may create only one.\
<br>
After each account is set up, take their access tokens and run: 
`synchro list <access_token>`
to get the catalog ids of the products.\
![img_18.png](img_18.png)
<br>
To get the location IDs for each account go to:
![img_17.png](img_17.png)
<br>
Then, make a `config.json` file in your working directory, and populate it with: 
```json
{
  "RealWorld": {
    "initial_value": 100,
    "platforms": [
      ["VendorA", {
        "Records": {
          "token": "...",
          "backoff": {
            "secs": 0,
            "nanos": 200000000
          },
          "target": ["<location_id>", "<product_id>"],
          "calibration_target": ["<location_id>", "<product_id>"],
        }
      }],
      ["VendorB", {
        "Polling": {
          "token":"...",
          "backoff": {
            "secs": 0,
            "nanos": 200000000
          },
          "target": ["<location_id>", "<product_id>"],
          "interpretation": "Transition" // How to interpret polls- Transition, Mutation, or Assignment
        }
      }]
    ]
  }
}
```
Now you are ready to run- run the command:
`synchro run <your_config_file>`
To test- you will need to open a second incognito window. Login to your developer account again, and go to the accounts page. There you can select a second account simultaneously.\
<br>
You can trigger stock  changes through the manage stock panel as shown previously- changes do not appear in the dashboard until a refresh.\


