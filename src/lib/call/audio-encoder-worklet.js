class AudioEncoderWorklet extends AudioWorkletProcessor {
    sampleCount = 0;

    process(inputs) {
        if (inputs.length === 0 || inputs[0].length === 0) {
            return true;
        }

        const channels = inputs[0];

        this.port.postMessage({
            timestamp: (this.sampleCount / sampleRate) * 1_000_000,
            channels: channels.map((channel) => channel.slice())
        });

        this.sampleCount += channels[0].length;
        return true;
    }
}

registerProcessor("audio-encoder-worklet", AudioEncoderWorklet);
