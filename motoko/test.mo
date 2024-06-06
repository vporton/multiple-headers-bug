import Types "HttpTypes";
import Text "mo:base/Text";
import Iter "mo:base/Iter";
import Blob "mo:base/Blob";
import Cycles "mo:base/ExperimentalCycles";

actor {
    public shared func test() : async (Text, [{name: Text; value: Text}]) {

        let ic : Types.IC = actor ("aaaaa-aa");

        let request : Types.HttpRequestArgs = {
            url = "https://local.vporton.name:8081";
            headers = [ {name = "x-my"; value = "A"}, {name = "x-my"; value = "B"} ];
            body = null;
            method = #get;
            max_response_bytes = ?10_000;
            transform = ?{ function = transform; context = "" };
        };

        Cycles.add<system>(20_000_000);
        let response : Types.HttpResponsePayload = await ic.http_request(request);

        let body2 = switch (Text.decodeUtf8(Blob.fromArray(response.body))) {
            case (?body) body;
            case null "";
        };
        (body2, response.headers);
    };

    public query func transform(args: Types.TransformArgs): async Types.HttpResponsePayload {
        // To be sure that something like different dates in the headers does not make them different,
        // remove all headers except of `x-my`:
        let myHeaders = Iter.filter<{name: Text; value: Text}>(args.response.headers.vals(), func (c: {name: Text; value: Text}) { c.name == "x-my" });
        {
            status = args.response.status;
            headers = Iter.toArray(myHeaders);
            body = args.response.body;
        };
    };

}