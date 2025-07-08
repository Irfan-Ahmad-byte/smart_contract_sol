# Litecoin Setup

To setup litecoin, you need to modify the litecoin.conf

```shell
sudo nano ~/.litecoin/litecoin.conf
```

and add following line

```shell
walletnotify=~/.litecoin/transaction.sh %s

```

after saving it

```shell
sudo nano ~/.litecoin/walletnotify.sh
```

and put following lines

```shell
#!/bin/sh
curl -d "txid=$1" http://127.0.0.1/some/route
```

after that

```shell
sudo chmod +x ~/.litecoin/walletnotify.sh

```

If you get error about file does not exist then following might be the case

1. You sudoed and user was switched to root so the file is expected to be in /root/.litecoin/walletnotify.sh
2. Check the /home/{username here}/.litecoin for the file to exist, for example /home/ubuntu/.litecoin
3. 

