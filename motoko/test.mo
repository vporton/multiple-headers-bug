import Types "HttpTypes";
import Text "mo:base/Text";
import Blob "mo:base/Blob";

actor {
    public shared func test() : async Text {

        let ic : Types.IC = actor ("aaaaa-aa");

        let request : Types.HttpRequestArgs = {
            url = "https://local.vporton.name:8081";
            headers = [ {name = "x-my"; value = "A"}, {name = "x-my"; value = "B"} ];
            body = null;
            method = #get;
            max_response_bytes = ?10_000;
            transform = ?{ function = transform; context = "" };
        };

        let response : Types.HttpResponsePayload = await ic.http_request(request);

        switch (Text.decodeUtf8(Blob.fromArray(response.body))) {
            case (?body) body;
            case null "";
        };
    };

    public query func transform(args: Types.TransformArgs): async Types.HttpResponsePayload {
        {
            status = args.response.status;
            headers = [];
            body = args.response.body;
        };
    };

}