# data_bot
## A natural language SQLite inperperter.
##### IMPORTANT: you will need to aquile an API key from openai to use this program.
data bot connects to sqlite databases and uses gpt-3.5 to construct sql queries from natural language.

[Screencast from 11-01-2023 08:51:16 AM.webm](https://github.com/Strange-Knoll/data_bot/assets/120497873/6ed67382-d35f-4172-ad6f-910327c14086)


### quick setup
- install rust
- clone the repo
- copy and paste your key into logs/api_key.txt
- run ```cargo run```

you will need to enter the path to your database when propmted, this path should not include the ```sqlite://``` prefix
once the database has successfully connected, ask your questions and start exploring the database.
