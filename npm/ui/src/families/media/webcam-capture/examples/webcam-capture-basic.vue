<!-- Camera booth with start, stop, switch, and countdown shutter controls; it renders idle and only requests the camera when the user presses Start. -->
<script setup lang="ts">
import {
  WebcamCapture,
  WebcamCaptureDeviceSelect,
  WebcamCapturePhoto,
  WebcamCaptureShutter,
  WebcamCaptureStartButton,
  WebcamCaptureStatusMessage,
  WebcamCaptureStopButton,
  WebcamCaptureSwitchCamera,
  WebcamCaptureVideo,
} from "../webcam-capture.ts";
</script>

<template>
  <WebcamCapture capture-type="image/jpeg" :capture-quality="0.9">
    <WebcamCaptureVideo />
    <WebcamCaptureStartButton />
    <WebcamCaptureStopButton />
    <WebcamCaptureSwitchCamera />
    <WebcamCaptureDeviceSelect />
    <WebcamCaptureShutter :countdown="3" />
    <WebcamCaptureStatusMessage />
    <WebcamCapturePhoto alt="Captured photo" />
  </WebcamCapture>
</template>
