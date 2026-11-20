export const reviewedScorecardTransitions: Readonly<
  Record<
    string,
    {
      readonly ruleId: string;
      readonly sourceSha256: string;
      readonly packetSha256: string;
      readonly nativePacketSha256: string;
      readonly runtimeVersion: string;
    }
  >
> = {
  "bare-slot-shorthand": {
    ruleId: "vue/v-slot-style",
    sourceSha256: "4b7c77d382e1d3e46eeb6765eb6ab6cfaebb7c1d2b7a2a65a06871289b734c6e",
    packetSha256: "c9ecb9b4792dd220f61315e47a3ea407d0a13e0ca8fe48c933d00a562bee55c1",
    nativePacketSha256: "d61b27a83593bdd6f5ed078f954af6e908ed2ceecf81ae11fa268e37ae270dd9",
    runtimeVersion: "10.9.2",
  },
  "class-decorator-prop": {
    ruleId: "vue/no-mutating-props",
    sourceSha256: "0e8ca44978bf438cf9ff9df7bb2adf315af226efc32b511d12a10c75dc22f6e1",
    packetSha256: "2f957e4d7659b623802a5e518ff5cf6b004f90f58759e7cf02981d9f56b9e083",
    nativePacketSha256: "208dbdcd5ae68be6b45d5eda85c39ff0e77b9e528dd50274659f50237094c3a0",
    runtimeVersion: "10.9.2",
  },
  "bare-slot-shorthand-scorecard": {
    ruleId: "vue/v-slot-style",
    sourceSha256: "4b7c77d382e1d3e46eeb6765eb6ab6cfaebb7c1d2b7a2a65a06871289b734c6e",
    packetSha256: "47819ba2c93bbf1cc4d131054bca29a9ef2dfa18fee6c4f15f9a0d4f5417b362",
    nativePacketSha256: "8873508e48adc3b555e20239e65ce3c8e513b9f8eea8026490daa2eb2e1ecae6",
    runtimeVersion: "10.12.0",
  },
  "class-decorator-prop-scorecard": {
    ruleId: "vue/no-mutating-props",
    sourceSha256: "0e8ca44978bf438cf9ff9df7bb2adf315af226efc32b511d12a10c75dc22f6e1",
    packetSha256: "b5d67aac101eef5eaded5f2b4920f68451eadc479ebf7605a12eaa0c40c38750",
    nativePacketSha256: "fd06e9e496c71c5578f79449a8a9668c04f5d4fdb078c9082a0ca4de986cba93",
    runtimeVersion: "10.12.0",
  },
};
