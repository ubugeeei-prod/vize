import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import type { Vector2 } from '@vueuse/gesture'
import type { mastodon } from 'masto'
import { useGesture } from '@vueuse/gesture'
import { useReducedMotion } from '@vueuse/motion'
const slideGap = 20
const doubleTapThreshold = 250

export default /*@__PURE__*/_defineComponent({
  __name: 'ModalMediaPreviewCarousel',
  props: {
    media: { type: Array, required: false, default: [] },
    "modelValue": { required: true }
  },
  emits: ["close", "update:modelValue"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const modelValue = _useModel(__props, "modelValue")
const view = ref()
const slider = ref()
const slide = ref()
const image = ref()
const reduceMotion = import.meta.server ? ref(false) : useReducedMotion()
const isInitialScrollDone = useTimeout(350)
const canAnimate = computed(() => isInitialScrollDone.value && !reduceMotion.value)
const scale = ref(1)
const x = ref(0)
const y = ref(0)
const isDragging = ref(false)
const isPinching = ref(false)
const maxZoomOut = ref(1)
const isZoomedIn = computed(() => scale.value > 1)
const enableAutoplay = usePreferences('enableAutoplay')
function goToFocusedSlide() {
  scale.value = 1
  x.value = slide.value[modelValue.value].offsetLeft * scale.value
  y.value = 0
}
onMounted(() => {
  const slideGapAsScale = slideGap / view.value.clientWidth
  maxZoomOut.value = 1 - slideGapAsScale
  goToFocusedSlide()
})
watch(modelValue, goToFocusedSlide)
let lastOrigin = [0, 0]
let initialScale = 0
useGesture({
  onPinch({ first, initial: [initialDistance], movement: [deltaDistance], da: [distance], origin, touches }) {
    isPinching.value = true
    if (first) {
      initialScale = scale.value
    }
    else {
      if (touches === 0)
        handleMouseWheelZoom(initialScale, deltaDistance, origin)
      else
        handlePinchZoom(initialScale, initialDistance, distance, origin)
    }
    lastOrigin = origin
  },
  onPinchEnd() {
    isPinching.value = false
    isDragging.value = false
    if (!isZoomedIn.value)
      goToFocusedSlide()
  },
  onDrag({ movement, delta, pinching, tap, last, swipe, event, xy }) {
    event.preventDefault()
    if (pinching)
      return
    if (last)
      handleLastDrag(tap, swipe, movement, xy)
    else
      handleDrag(delta, movement)
  },
}, {
  domTarget: view,
  eventOptions: {
    passive: false,
  },
})
const shiftRestrictions = computed(() => {
  const focusedImage = image.value[modelValue.value]
  const focusedSlide = slide.value[modelValue.value]

  const scaledImageWidth = focusedImage.offsetWidth * scale.value
  const scaledHorizontalOverflow = scaledImageWidth / 2 - view.value.clientWidth / 2 + slideGap
  const horizontalOverflow = Math.max(0, scaledHorizontalOverflow / scale.value)

  const scaledImageHeight = focusedImage.offsetHeight * scale.value
  const scaledVerticalOverflow = scaledImageHeight / 2 - view.value.clientHeight / 2 + slideGap
  const verticalOverflow = Math.max(0, scaledVerticalOverflow / scale.value)

  return {
    left: focusedSlide.offsetLeft - horizontalOverflow,
    right: focusedSlide.offsetLeft + horizontalOverflow,
    top: focusedSlide.offsetTop - verticalOverflow,
    bottom: focusedSlide.offsetTop + verticalOverflow,
  }
})
function handlePinchZoom(initialScale: number, initialDistance: number, distance: number, [originX, originY]: Vector2) {
  scale.value = initialScale * (distance / initialDistance)
  scale.value = Math.max(maxZoomOut.value, scale.value)
  const deltaCenterX = originX - lastOrigin[0]
  const deltaCenterY = originY - lastOrigin[1]
  handleZoomDrag([deltaCenterX, deltaCenterY])
}
function handleMouseWheelZoom(initialScale: number, deltaDistance: number, [originX, originY]: Vector2) {
  scale.value = initialScale + (deltaDistance / 1000)
  scale.value = Math.max(maxZoomOut.value, scale.value)
  const deltaCenterX = lastOrigin[0] - originX
  const deltaCenterY = lastOrigin[1] - originY
  handleZoomDrag([deltaCenterX, deltaCenterY])
}
function handleLastDrag(tap: boolean, swipe: Vector2, movement: Vector2, position: Vector2) {
  isDragging.value = false
  if (tap)
    handleTap(position)
  else if (swipe[0] || swipe[1])
    handleSwipe(swipe, movement)
  else if (!isZoomedIn.value)
    slideToClosestSlide()
}
let lastTapAt = 0
function handleTap([positionX, positionY]: Vector2) {
  const now = Date.now()
  const isDoubleTap = now - lastTapAt < doubleTapThreshold
  lastTapAt = now
  if (!isDoubleTap)
    return
  if (isZoomedIn.value) {
    goToFocusedSlide()
  }
  else {
    const focusedSlideBounding = slide.value[modelValue.value]?.getBoundingClientRect()
    if (focusedSlideBounding) {
      const slideCenterX = focusedSlideBounding.left + focusedSlideBounding.width / 2
      const slideCenterY = focusedSlideBounding.top + focusedSlideBounding.height / 2
      scale.value = 3
      x.value += positionX - slideCenterX
      y.value += positionY - slideCenterY
      restrictShiftToInsideSlide()
    }
  }
}
function handleSwipe([horiz, vert]: Vector2, [movementX, movementY]: Vector2) {
  if (isZoomedIn.value || isPinching.value)
    return
  const isHorizontalDrag = Math.abs(movementX) >= Math.abs(movementY)
  if (isHorizontalDrag) {
    if (horiz === 1) // left
      modelValue.value = Math.max(0, modelValue.value - 1)
    if (horiz === -1) // right
      modelValue.value = Math.min(__props.media.length - 1, modelValue.value + 1)
  }
  else if (vert === 1 || vert === -1) {
    emit('close')
  }
  goToFocusedSlide()
}
function slideToClosestSlide() {
  const startOfFocusedSlide = slide.value[modelValue.value].offsetLeft * scale.value
  const slideWidth = slide.value[modelValue.value].offsetWidth * scale.value
  if (x.value > startOfFocusedSlide + slideWidth / 2)
    modelValue.value = Math.min(__props.media.length - 1, modelValue.value + 1)
  else if (x.value < startOfFocusedSlide - slideWidth / 2)
    modelValue.value = Math.max(0, modelValue.value - 1)
  goToFocusedSlide()
}
function handleDrag(delta: Vector2, movement: Vector2) {
  isDragging.value = true
  if (isZoomedIn.value)
    handleZoomDrag(delta)
  else
    handleSlideDrag(movement)
}
function handleZoomDrag([deltaX, deltaY]: Vector2) {
  x.value -= deltaX / scale.value
  y.value -= deltaY / scale.value
  restrictShiftToInsideSlide()
}
function handleSlideDrag([movementX, movementY]: Vector2) {
  goToFocusedSlide()
  if (Math.abs(movementY) > Math.abs(movementX)) // vertical movement is more than horizontal
    y.value -= movementY / scale.value
  else
    x.value -= movementX / scale.value
  if (__props.media.length === 1)
    x.value = 0
}
function restrictShiftToInsideSlide() {
  x.value = Math.min(shiftRestrictions.value.right, Math.max(shiftRestrictions.value.left, x.value))
  y.value = Math.min(shiftRestrictions.value.bottom, Math.max(shiftRestrictions.value.top, y.value))
}
const sliderStyle = computed(() => {
  const style = {
    transform: `scale(${scale.value}) translate(${-x.value}px, ${-y.value}px)`,
    transition: 'none',
    gap: `${slideGap}px`,
  }

  if (canAnimate.value && !isDragging.value && !isPinching.value)
    style.transition = 'all 0.3s ease'

  return style
})
const imageStyle = computed(() => ({
  cursor: isDragging.value ? 'grabbing' : 'grab',
}))

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "view", ref: view,
      flex: "",
      "flex-row": "",
      "h-full": "",
      "w-full": "",
      "overflow-hidden": ""
    }, [ _createElementVNode("div", {
        ref_key: "slider", ref: slider,
        style: _normalizeStyle(sliderStyle.value),
        "w-full": "",
        "h-full": "",
        flex: "",
        "items-center": ""
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.media, (item) => {
          return (_openBlock(), _createElementBlock("div", {
            key: item.id,
            ref: "slide",
            "flex-shrink-0": "",
            "w-full": "",
            "h-full": "",
            flex: "",
            "items-center": "",
            "justify-center": ""
          }, [
            _createVNode(_resolveDynamicComponent(item.type === 'gifv' ? 'video' : 'img'), {
              ref_key: "image", ref: image,
              autoplay: _unref(enableAutoplay),
              controls: "",
              loop: "",
              "select-none": "",
              "max-w-full": "",
              "max-h-full": "",
              style: _normalizeStyle(imageStyle.value),
              draggable: false,
              src: item.url || item.previewUrl,
              alt: item.description || ''
            }, null, 12 /* STYLE, PROPS */, ["autoplay", "draggable", "src", "alt"])
          ], 512 /* NEED_PATCH */))
        }), 128 /* KEYED_FRAGMENT */)) ], 4 /* STYLE */) ], 512 /* NEED_PATCH */))
}
}

})
