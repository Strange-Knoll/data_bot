# data_bot
## A natural language SQLite inperperter.
##### IMPORTANT: you will need to aquile an API key from openai to use this program.
data bot connects to sqlite databases and uses gpt-3.5 to construct sql queries from natural language.

[Screencast from 11-02-2023 04:43:58 PM.webm](https://github.com/Strange-Knoll/data_bot/assets/120497873/7e0291aa-fe69-4d3a-8cbb-52cafd37a0dc)


### quick setup
- install rust
- clone the repo
- copy and paste your key into logs/api_key.txt
- run ```cargo run```

you will need to enter the path to your database when propmted, this path should not include the ```sqlite://``` prefix
once the database has successfully connected, ask your questions and start exploring the database.

chat_bot contains an ai error handleing system, it is currently possible for the ai to become stuck in a correcting loop.
if you encounter a never ending loop abort the application with ```ctrl-c```
