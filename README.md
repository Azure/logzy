# Logzy

Simple command-line log formatter to make structured JSON logs more readable for humans.

Outputs "standard" fields in a fixed order, followed by any remaining fields.

Standard fields:

- `ts`: timestamp formatted in a readable way
- `level`: INFO, WARN ERRO...
- `component`: HRE, PLC...
- `msg`: Main log message

Without logzy:
```
{"msg":"Starting HRE server","level":"INFO","ts":"2017-10-12T16:37:19.996843704+01:00","component":"HRE"}
{"msg":"HRE listening on http://127.0.0.1:8444 with 1 thread.","level":"INFO","ts":"2017-10-12T16:37:20.000958956+01:00","component":"HRE"}
{"msg":"FGF address","level":"INFO","ts":"2017-10-12T16:37:20.001062480+01:00","component":"HRE","fgf_addr":"http://127.0.0.1:1234"}
{"msg":"TET address","level":"INFO","ts":"2017-10-12T16:37:20.001101925+01:00","component":"HRE","tet_addr":"http://127.0.0.1:1234"}
{"msg":"Request::new: addr=127.0.0.1:43038, \"GET /hre/v1/infra/up HTTP/1.1\"","level":"INFO","ts":"2017-10-12T16:37:21.042250435+01:00"}
{"msg":"Received HTTP request","level":"INFO","ts":"2017-10-12T16:37:21.042872981+01:00","span_id":"378a9ff8-b154-4eae-a2a0-f67b056a93d9","component":"HRE","remote_addr":"127.0.0.1:43038","headers":"Host: 127.0.0.1:8444\r\nUser-Agent: reqwest/0.7.3\r\nAccept: */*\r\nAccept-Encoding: gzip\r\n","uri":"/wsd/v1/infra/up","method":"GET"}
{"msg":"Received CheckMicroserviceUp request","level":"INFO","ts":"2017-10-12T16:37:21.043045112+01:00","span_id":"378a9ff8-b154-4eae-a2a0-f67b056a93d9","component":"HRE"}
{"msg":"Send response","level":"INFO","ts":"2017-10-12T16:37:21.043099586+01:00","span_id":"378a9ff8-b154-4eae-a2a0-f67b056a93d9","component":"HRE","rsp":"Response { status: NoContent, version: Http11, headers: {} }"}
{"msg":"Send HTTP response","level":"INFO","ts":"2017-10-12T16:37:21.043518551+01:00","span_id":"378a9ff8-b154-4eae-a2a0-f67b056a93d9","component":"HRE","headers":"","status":"204 No Content"}
```

With logzy (with colouring on a real terminal):
```
16:36:15.476 INFO  HRE Starting HRE server
16:36:15.480 INFO  HRE HRE listening on http://127.0.0.1:8444 with 1 thread.
16:36:15.480 INFO  HRE FGF address fgf_addr:"http://127.0.0.1:1234"
16:36:15.480 INFO  HRE TET address tet_addr:"http://127.0.0.1:1234"
16:36:16.531 INFO      Request::new: addr=127.0.0.1:42910, "GET /hre/v1/infra/up HTTP/1.1"
16:36:16.532 INFO  HRE Received HTTP request headers:"Host: 127.0.0.1:8444\r\nUser-Agent: reqwest/0.7.3\r\nAccept: */*\r\nAccept-Encoding: gzip\r\n" method:"GET" remote_addr:"127.0.0.1:42910" span_id:"6b87e7a9-ae74-4ab9-af01-938514c8f710" uri:"/wsd/v1/infra/up"
16:36:16.532 INFO  HRE Received CheckMicroserviceUp request span_id:"6b87e7a9-ae74-4ab9-af01-938514c8f710"
16:36:16.532 INFO  HRE Send response rsp:"Response { status: NoContent, version: Http11, headers: {} }" span_id:"6b87e7a9-ae74-4ab9-af01-938514c8f710"
16:36:16.533 INFO  HRE Send HTTP response headers:"" span_id:"6b87e7a9-ae74-4ab9-af01-938514c8f710" status:"204 No Content"
```

## How to run

With logs from `kubectl`:
```bash
kubectl logs -f <service-name> | logzy
```

With logs from `docker`:
```bash
sudo docker logs -f <container> | logzy
```

With logs from a file:
```bash
cat <file> | logzy
```
or
```bash
tail -f <file> | logzy
```

To strip out any unwanted logs, use `grep -v`.  For example to remove all `DEBG` logs:
```shell
sudo docker logs <container> | grep -v DEBG | logzy
```

## Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit https://cla.opensource.microsoft.com.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
