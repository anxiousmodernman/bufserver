# bufserver 

A little tcp server that holds a ring buffer of what you send to it, and serves
the contents of that buffer over a unix socket. Only works with unicode text,
not binary.

## Usage

Run the server. It runs on port 9595.

```
bufserver
```

Connect to the server with a tcp client and send it some stuff.

```
nc locahost 9595   # then type something
```

bufserver will be holding up to 5MB of unicode text in a ringbuffer.

Use the amazing `socat` program (or something else you've cooked up) to read the
contents of the buffer by connecting to a unix socket at */tmp/bufserver*. 
Repeated reads will not clear the buffer, so it's okay to run this multiple times.

```
socat -u UNIX-CONNECT:/tmp/bufserver - | less
```


