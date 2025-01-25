Hyper

# HTTP Body
An HTTP body is a stream of Frames, each Frame containing parts of the Body data. So rather than reading the entire Body into a buffer before sending our response, we can stream each frame as it arrives. 

Buffering usually refers to collecting multiple frames or chunks of data into memory (e.g., in a Vec or Bytes) before processing them. This involves storing the data temporarily in a buffer until the complete response (or a significant part of it) is available.
Latency Reduction:
The server can start sending back processed data (e.g., the uppercase bytes) immediately after receiving the first frame, reducing overall response time.

Body mapping
Every data Frame of our body stream is a chunk of bytes, which we can conveniently represent using the Bytes type from hyper. It can be easily converted into other typical containers of bytes.

Buffering the Request Body
What if we want our echo service to reverse the data it received and send it back to us? We can’t really stream the data as it comes in, since we need to find the end before we can respond. To do this, we can explore how to easily collect the full body.

# Box<dyn std::error::Error + Send + Sync>
Send: Indicates the error can safely be transferred across thread boundaries.
Sync: Indicates the error can safely be accessed concurrently from multiple threads.

When tokio::task::spawn is used, the tasks are sent to a worker thread pool for execution. For this to happen safely, any data shared or passed between threads must implement Send (to be sent to another thread) and sometimes Sync (to allow concurrent access).

# Graceful Shutdown
is when a connection stops allowing new requests, while allowing currently in-flight requests to complete.
need several pieces:
A signal for when to start the shutdown.
An accept loop handling newly received connections.
A watcher to coordinate the shutdown.