import { tables } from "$lib/module_bindings";
import type { CallFrame, CallFrameSource } from "$lib/module_bindings/types";
import { DbConnectionImpl, Identity, toSql } from "spacetimedb";
import type { ConnectionState } from "spacetimedb/angular";
import { getContext, setContext } from "svelte";
import { SvelteMap } from "svelte/reactivity";
import { writable, type Readable, type Writable } from "svelte/store";

const DECODER_CONTEXT_KEY = "decoder";
const AUDIO_BUFFER_DELAY = 0.02;

export function createDecoderProvider(conn: Writable<ConnectionState>) {
    const store = writable<Decoder | undefined>(undefined);

    conn.subscribe((conn) => {
        const connection = conn.getConnection();
        if (connection?.isActive) store.set(new Decoder(connection));
    });

    setContext(DECODER_CONTEXT_KEY, store);
}

export function useDecoder(): Readable<Decoder | undefined> {
    return { subscribe: getContext<Writable<Decoder | undefined>>(DECODER_CONTEXT_KEY).subscribe };
}

export class Decoder {
    #video_decoders: SvelteMap<
        string,
        {
            decoder: VideoDecoder;
            canvas: HTMLCanvasElement;
            has_key_frame: boolean;
            timeout?: number;
        }
    >;
    #audio_decoders: SvelteMap<
        string,
        {
            decoder: AudioDecoder;
            gain_node: GainNode;
            has_key_frame: boolean;
            timeout?: number;
        }
    >;

    constructor(conn: DbConnectionImpl<any>) {
        conn.db[tables.call_frames.accessorName].onInsert((_, call_frame) =>
            this.#handleCallFrame(call_frame as CallFrame)
        );
        conn.subscriptionBuilder().subscribe(toSql(tables.call_frames));

        this.#video_decoders = new SvelteMap();
        this.#audio_decoders = new SvelteMap();
    }

    #handleCallFrame(call_frame: CallFrame) {
        if (call_frame.frameType.tag == "Video") this.#decodeVideoChunk(call_frame);
        if (call_frame.frameType.tag == "Audio") this.#decodeAudioChunk(call_frame);
    }

    async #decodeVideoChunk(call_frame: CallFrame) {
        if (call_frame.frameType.tag != "Video") return;

        const key = `${call_frame.frameSource.tag}:${call_frame.sender.toString()}`;

        let decoder = this.#video_decoders.get(key);

        if (decoder === undefined) {
            const canvas = document.createElement("canvas");
            const context = canvas.getContext("2d", {
                alpha: false
            }) as CanvasRenderingContext2D;

            decoder = {
                decoder: new VideoDecoder({
                    output: (video_frame) => {
                        canvas.height = video_frame.displayHeight;
                        canvas.width = video_frame.displayWidth;

                        context.drawImage(video_frame, 0, 0);
                        video_frame.close();
                    },
                    error: console.error
                }),
                canvas,
                has_key_frame: false
            };
            decoder.decoder.configure({ codec: call_frame.codec });
            this.#video_decoders.set(key, decoder);
        }

        if ((decoder.has_key_frame ||= call_frame.chunkType.tag == "Key"))
            decoder?.decoder.decode(
                new EncodedVideoChunk({
                    type: call_frame.chunkType.tag == "Key" ? "key" : "delta",
                    data: call_frame.data,
                    timestamp: call_frame.timestamp
                })
            );

        if (decoder.timeout) clearTimeout(decoder.timeout);

        decoder.timeout = setTimeout(() => this.#video_decoders.delete(key), 2000);
    }

    async #decodeAudioChunk(call_frame: CallFrame) {
        if (call_frame.frameType.tag != "Audio") return;

        const key = `${call_frame.frameSource.tag}:${call_frame.sender.toString()}`;

        let decoder = this.#audio_decoders.get(key);

        if (decoder === undefined) {
            const key = `${call_frame.frameSource.tag}:${call_frame.sender.toString()}`;
            const context = new AudioContext({
                sampleRate: call_frame.frameType.value.sampleRate,
                latencyHint: "interactive"
            });
            const gain_node = context.createGain();

            let base_timestamp: null | number = null;
            let base_time: null | number = null;

            if (localStorage.getItem(`Gain:${key}`))
                gain_node.gain.value = parseFloat(localStorage.getItem(`Gain:${key}`) || "");

            gain_node.connect(context.destination);

            decoder = {
                decoder: new AudioDecoder({
                    output: (audio_data) => {
                        const audio_buffer = context.createBuffer(
                            audio_data.numberOfChannels,
                            audio_data.numberOfFrames,
                            audio_data.sampleRate
                        );

                        for (let index = 0; index < audio_data.numberOfChannels; index++) {
                            const channel = new Float32Array(audio_data.numberOfFrames);

                            audio_data.copyTo(channel, { planeIndex: index, format: "f32" });
                            audio_buffer.copyToChannel(channel, index);
                        }

                        const source = context.createBufferSource();

                        source.buffer = audio_buffer;
                        source.loop = false;
                        source.connect(gain_node);

                        if (base_timestamp === null || base_time == null) {
                            base_timestamp = audio_data.timestamp;
                            base_time = context.currentTime + AUDIO_BUFFER_DELAY;
                        }

                        const start_time =
                            base_time + (audio_data.timestamp - base_timestamp) / 1_000_000;

                        if (start_time >= context.currentTime) source.start(start_time);

                        audio_data.close();
                    },
                    error: console.error
                }),
                gain_node,
                has_key_frame: false
            };
            decoder.decoder.configure({
                codec: call_frame.codec,
                sampleRate: call_frame.frameType.value.sampleRate,
                numberOfChannels: call_frame.frameType.value.numberOfChannels
            });
            this.#audio_decoders.set(key, decoder);
        }

        if ((decoder.has_key_frame ||= call_frame.chunkType.tag == "Key"))
            decoder.decoder.decode(
                new EncodedAudioChunk({
                    type: call_frame.chunkType.tag == "Key" ? "key" : "delta",
                    data: call_frame.data,
                    timestamp: call_frame.timestamp
                })
            );

        if (decoder.timeout) clearTimeout(decoder.timeout);

        decoder.timeout = setTimeout(() => this.#audio_decoders.delete(key), 100);
    }

    getVideo(identity: Identity, frame_source: CallFrameSource) {
        const key = `${frame_source.tag}:${identity.toString()}`;
        const decoder = this.#video_decoders.get(key);

        return (
            decoder && {
                canvas: decoder.canvas
            }
        );
    }

    getAudio(identity: Identity, frame_source: CallFrameSource) {
        const key = `${frame_source.tag}:${identity.toString()}`;
        const decoder = this.#audio_decoders.get(key);

        return (
            decoder && {
                get gain() {
                    return decoder.gain_node.gain.value;
                },
                set gain(gain) {
                    decoder.gain_node.gain.value = gain;
                    localStorage.setItem(`Gain:${key}`, gain.toString());
                }
            }
        );
    }
}
