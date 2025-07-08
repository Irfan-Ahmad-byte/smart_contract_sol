Following are the requirements of data in Database to run the module properly:

1- At least one record should be added in the admin table, this is required for the admin user:
This admin is required to make requests to the Coins table.
admin table:

```
    id: int
    name: string (name of the admin)
    api_key: string (api key of the admin)
    api_secret: string (api secret of the admin)
    status: boolean (whether the admin is active or not)
    created_at: time
    updated_at: time
```

2- At least a coin should be added in the coins table, this is also requird for conversion rates:
coins table:

```
    id: int
    coin_name: string (name of the coin)
    symbol: string (symbol of the coin)
    status: boolean (whether the coin is active or not)
    created_at: time
    updated_at: time
```