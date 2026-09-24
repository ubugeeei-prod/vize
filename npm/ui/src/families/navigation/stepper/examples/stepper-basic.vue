<!-- Uncontrolled checkout stepper with a labelled step list and one panel per step. -->
<script setup lang="ts">
import {
  StepperContent,
  StepperItem,
  StepperList,
  StepperRoot,
  StepperTrigger,
} from "../stepper.ts";
</script>

<template>
  <StepperRoot default-value="shipping">
    <StepperList aria-label="Checkout steps">
      <StepperItem value="shipping" text-value="Shipping" completed>
        <StepperTrigger>Shipping</StepperTrigger>
      </StepperItem>
      <StepperItem value="billing" text-value="Billing">
        <StepperTrigger>Billing</StepperTrigger>
      </StepperItem>
      <StepperItem value="review" text-value="Review">
        <StepperTrigger>Review</StepperTrigger>
      </StepperItem>
    </StepperList>
    <StepperContent value="shipping">Enter your shipping address.</StepperContent>
    <StepperContent value="billing">Choose a payment method.</StepperContent>
    <StepperContent value="review">Confirm your order.</StepperContent>
  </StepperRoot>
</template>
