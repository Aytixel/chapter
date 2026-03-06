import { DbConnection } from "$lib/module_bindings";
import {
    CallFrameChunkType,
    CallFrameType,
    type CallFrameSource
} from "$lib/module_bindings/types";
import AudioEncoderWorklet from "./audio-encoder-worklet.js?raw";

const VIDEO_CODEC = "av01.0.12M.08";
const AUDIO_CODEC = "opus";
const DEFAULT_SAMPLE_RATE = 44100;
const DEFAULT_NUMBER_OF_CHANNELS = 2;

export class Encoder {
    #conn: DbConnection;
    #stream: MediaStream;
    #source: CallFrameSource;
    #video_encoder?: VideoEncoder;
    #audio_encoder?: AudioEncoder;

    constructor(conn: DbConnection | null, stream: MediaStream, source: CallFrameSource) {
        if (!conn) throw "Can't encode stream if there is no db connection";

        this.#conn = conn;
        this.#stream = stream;
        this.#source = source;

        const video_stream_tracks = stream.getVideoTracks();
        const audio_stream_tracks = stream.getAudioTracks();

        if (video_stream_tracks[0]) this.#registerVideoTrack(video_stream_tracks[0]);
        if (audio_stream_tracks[0]) this.#registerAudioTrack(audio_stream_tracks[0]);
    }

    stop() {
        if (this.#video_encoder && this.#video_encoder.state != "closed") {
            this.#video_encoder.close();
            this.#video_encoder = undefined;
        }
        if (this.#audio_encoder && this.#audio_encoder.state != "closed") {
            this.#audio_encoder.close();
            this.#audio_encoder = undefined;
        }

        for (const track of this.#stream.getTracks()) {
            track.stop();
        }
    }

    async #registerVideoTrack(track: MediaStreamTrack) {
        this.#video_encoder = new VideoEncoder({
            output: (chunk) => {
                const buffer = new ArrayBuffer(chunk.byteLength);

                chunk.copyTo(buffer);
                this.#conn.reducers.sendCallFrame({
                    frameSource: this.#source,
                    frameType: CallFrameType.Video,
                    codec: VIDEO_CODEC,
                    chunkType:
                        chunk.type == "key" ? CallFrameChunkType.Key : CallFrameChunkType.Delta,
                    timestamp: chunk.timestamp,
                    data: new Uint8Array(buffer)
                });
            },
            error: console.error
        });

        const settings = track.getSettings();
        const video = document.createElement("video");
        let keyFrameCount = -1;

        const process_frame = (timestamp: number) => {
            if (this.#video_encoder) {
                video.requestVideoFrameCallback(process_frame);

                const frame = new VideoFrame(video, {
                    timestamp: timestamp * 1_000,
                    alpha: "discard"
                });
                const newKeyFrameCount = Math.floor(timestamp / (1_000 * 2));

                this.#video_encoder.encode(frame, { keyFrame: newKeyFrameCount != keyFrameCount });

                keyFrameCount = newKeyFrameCount;

                frame.close();
            } else video.pause();
        };

        video.requestVideoFrameCallback(process_frame);
        video.srcObject = new MediaStream([track]);

        await new Promise((resolve) => (video.onloadedmetadata = resolve));

        this.#video_encoder.configure({
            codec: VIDEO_CODEC,
            height: video.videoHeight,
            width: video.videoWidth,
            framerate: Math.min(60, settings.frameRate || Infinity),
            latencyMode: "realtime",
            alpha: "discard"
        });

        await video.play();
    }

    async #registerAudioTrack(track: MediaStreamTrack) {
        const settings = track.getSettings();

        this.#audio_encoder = new AudioEncoder({
            output: (chunk) => {
                const buffer = new ArrayBuffer(chunk.byteLength);

                chunk.copyTo(buffer);
                this.#conn.reducers.sendCallFrame({
                    frameSource: this.#source,
                    frameType: CallFrameType.Audio({
                        sampleRate: settings.sampleRate || DEFAULT_SAMPLE_RATE,
                        numberOfChannels: settings.channelCount || DEFAULT_NUMBER_OF_CHANNELS
                    }),
                    codec: AUDIO_CODEC,
                    chunkType:
                        chunk.type == "key" ? CallFrameChunkType.Key : CallFrameChunkType.Delta,
                    timestamp: chunk.timestamp,
                    data: new Uint8Array(buffer)
                });
            },
            error: console.error
        });
        this.#audio_encoder.configure({
            codec: AUDIO_CODEC,
            sampleRate: settings.sampleRate || DEFAULT_SAMPLE_RATE,
            numberOfChannels: settings.channelCount || DEFAULT_NUMBER_OF_CHANNELS,
            opus: { application: "voip" }
        });

        const audio_context = new AudioContext({
            sampleRate: settings.sampleRate || DEFAULT_SAMPLE_RATE,
            latencyHint: "interactive"
        });
        const source = new MediaStreamAudioSourceNode(audio_context, {
            mediaStream: new MediaStream([track])
        });

        const blob = new Blob([AudioEncoderWorklet], { type: "application/javascript" });
        const worklet_url = URL.createObjectURL(blob);

        await audio_context.audioWorklet.addModule(worklet_url);

        const worklet_node = new AudioWorkletNode(audio_context, "audio-encoder-worklet", {
            numberOfInputs: 1,
            numberOfOutputs: 0,
            channelCount: settings.channelCount || DEFAULT_NUMBER_OF_CHANNELS
        });

        source.connect(worklet_node);

        worklet_node.port.onmessage = (event) => {
            if (!this.#audio_encoder) return;

            const { timestamp, channels }: { timestamp: number; channels: Float32Array[] } =
                event.data;

            const data = new Float32Array(channels[0].length * channels.length);

            for (let i = 0; i < channels.length; i++) {
                data.set(channels[i], i * channels[0].length);
            }

            const audio_data = new AudioData({
                format: "f32-planar",
                sampleRate: audio_context.sampleRate,
                numberOfFrames: channels[0].length * channels.length,
                numberOfChannels: settings.channelCount || DEFAULT_NUMBER_OF_CHANNELS,
                timestamp,
                data
            });

            this.#audio_encoder.encode(audio_data);

            audio_data.close();
        };
    }
}
