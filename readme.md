mogmod-backend

backend component of the larger mogmod project. This system reads messages from chat applications (discord, twtich chat, ...), scores them with ML models (E.G. sentiment analysis, hate speech detection) available on [HuggingFace](https://huggingface.co/), then saves the relevant data to a postgresdb. This data can then be used to build dashboards or alerting for moderators (TBD)

uses microservice architecture communicating over REST APIs.

I am doing this for fun and have no specific goals or timelines to meet, so development/pushes to github will be sporadic.