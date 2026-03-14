import type { CallFrame } from "$lib/module_bindings/types";

let decoder: VideoDecoder | undefined;
let codec: string | undefined;
let has_key_frame = false;

export type VideoDecoderWorkerCommand =
    | {
          command: "configuration";
          canvas: OffscreenCanvas;
          codec: string;
      }
    | ({
          command: "frame";
      } & CallFrame);

onmessage = (e: MessageEvent<VideoDecoderWorkerCommand>) => {
    switch (e.data.command) {
        case "configuration":
            if (e.data.codec !== codec) {
                const canvas = e.data.canvas;
                const context = canvas.getContext("2d", {
                    alpha: false
                }) as OffscreenCanvasRenderingContext2D;

                decoder = new VideoDecoder({
                    output: (video_frame) => {
                        canvas.height = video_frame.displayHeight;
                        canvas.width = video_frame.displayWidth;

                        context.drawImage(video_frame, 0, 0);
                        video_frame.close();
                    },
                    error: console.error
                });
                decoder.configure({ codec: e.data.codec });

                codec = e.data.codec;
                has_key_frame = false;
            }
            break;
        case "frame":
            if (decoder && (has_key_frame ||= e.data.chunkType.tag == "Key")) {
                decoder.decode(
                    new EncodedVideoChunk({
                        type: e.data.chunkType.tag == "Key" ? "key" : "delta",
                        data: e.data.data,
                        timestamp: e.data.timestamp
                    })
                );
            }
            break;
    }
};
