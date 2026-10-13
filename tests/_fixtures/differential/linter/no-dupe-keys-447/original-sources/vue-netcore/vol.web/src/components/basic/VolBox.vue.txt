<template>
  <div class="vol-dialog">
    <el-dialog v-model="vmodel" :destroy-on-close="destroyOnClose" :close-on-click-modal="false"
      :close-on-press-escape="false" :width="boxWidth" :fullscreen="fullscreen"
      :draggable="draggable || $global.boxDraggable" :modal="modal" :before-close="handleClose">
      <template #header>
        <i :class="icon"></i> {{ $ts(title) }}
        <slot name="header"></slot>

        <button v-if="showFullscreen && showFull" class="el-dialog__headerbtn" type="button"
          style="right: 65px; color: var(--el-color-info)" @click="handleFullScreen">
          <i class="el-icon el-icon-full-screen" :title="$ts('最大化')"></i>
        </button>
        <button class="el-dialog__headerbtn" type="button" style="right: 35px; color: var(--el-color-info)"
          @click="minimizeClick">
          <i class="el-icon-minus" :title="$ts('最小化')"></i>
        </button>
      </template>
      <el-scrollbar :max-height="contentHeight">
        <div ref="dialogContent" v-if="inited" class="srcoll-content"
          :style="{ padding: padding + 'px', height: height ? height + 'px' : 'auto' }">
          <slot name="content"></slot>
          <slot></slot>
        </div>
      </el-scrollbar>
      <template #footer>
        <div class="dia-footer" v-if="footer">
          <slot name="footer"></slot>
          <el-button type="primary" v-if="!footer" size="mini" @click="handleClose()"><i class="el-icon-close"></i>{{
            $ts("关闭") }}</el-button>
        </div>
      </template>
    </el-dialog>
    <div class="minimize" v-if="minimize">
      <div class="fx-1"><i class="el-icon-warning-outline"></i>{{ $ts(title) }}</div>
      <div>
        <el-button type="default" text @click="restoreDialog">
          <i class="el-icon-copy-document" size="small" :title="$ts('还原')"></i></el-button>
        <el-button type="default" text @click="minimizeCloseClick">
          <i class="el-icon-close" :title="$ts('关闭')"></i></el-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  getCurrentInstance,
  ref,
  watch,
  useSlots,
} from "vue";

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false,
  },
  lazy: {
    //是否开启懒加载2020.12.06
    type: Boolean,
    default: false,
  },
  icon: {
    type: String,
    default: "el-icon-warning-outline",
  },
  title: {
    type: String,
    default: "基本信息",
  },
  height: {
    type: Number,
    default: 0,
  },
  width: {
    type: Number,
    default: 650,
  },
  padding: {
    type: Number,
    default: 16,
  },
  modal: {
    //是否需要遮罩层
    type: Boolean,
    default: true,
  },
  draggable: {
    //启用可拖拽功能
    type: Boolean,
    default: false,
  },
  mask: {
    type: Boolean,
    default: true,
  },
  onModelClose: {
    //2021.07.11增加弹出框关闭事件
    type: Function,
    default: (iconClick) => {
      return true;
    },
  },
  footer: {
    //是否显示底部按钮
    type: Boolean,
    default: true,
  },
  full: {
    type: Boolean,
    default: false,
  },
  showFull: {
    type: Boolean,
    default: true,
  },
  destroyOnClose: {
    //当关闭 Dialog 时，销毁其中的元素
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(["update:modelValue", "fullscreen"]);

const clientFullHeight = document.body.clientHeight;
const clientHeight = clientFullHeight * 0.95 - 60;
const minimize = ref(false);
const inited = ref(true);
const vmodel = ref(props.modelValue);
const footer = ref(false);
const top = ref(100);

const slots = useSlots();
footer.value = !!slots.footer;

const contentHeight = ref(props.height || 200);

const handleClose = (done, iconClose) => {
  const result = props.onModelClose(!!iconClose);
  if (result === false) return;
  vmodel.value = false;
  emit("update:modelValue", false);
  done && done();
};

const calcHeight = (val) => {
  //  if (props.height > clientHeight) {
  //  if(true){
  contentHeight.value = clientHeight - 30;
  return clientHeight / -2 + "px";
  //  }
  // contentHeight.value = val || props.height;
  // return (props.height + 56) / -2 + 'px';
};

top.value = calcHeight();

const boxWidth = ref(props.width);

watch(
  () => props.modelValue,
  (newVal) => {
    vmodel.value = newVal;
    if (newVal) {
      minimize.value = false;
    }
  }
);

watch(
  () => props.height,
  () => {
    top.value = calcHeight();
  }
);

watch(
  () => props.width,
  (newVal) => {
    boxWidth.value = newVal;
  }
);

const { proxy } = getCurrentInstance();
const fullscreen = ref(false);
const showFullscreen = ref(true);

if (typeof proxy.$global.fullscreen === "boolean") {
  showFullscreen.value = proxy.$global.fullscreen;
}
fullscreen.value = props.full;

watch(
  () => props.full,
  (newVal) => {
    fullscreen.value = newVal;
  }
);

const dialogContent = ref();
let orgHeight = 0;
const handleFullScreen = () => {
  //全屏时获取非全屏的高度
  if (!fullscreen.value) {
    orgHeight = dialogContent.value.getBoundingClientRect().height
    contentHeight.value = clientFullHeight - 95
  }
  if (!orgHeight) {
    orgHeight = props.height;
    contentHeight.value = orgHeight
  }
  fullscreen.value = !fullscreen.value;
  emit("fullscreen", fullscreen.value, orgHeight, clientFullHeight - 95);
};

//通过全局计算minimize开启的数量来处理最小化的位置
const minimizeClick = () => {
  vmodel.value = false;
  minimize.value = true;
  emit("update:modelValue", false);
};

const restoreDialog = () => {
  minimize.value = false;
  vmodel.value = true;
  emit("update:modelValue", true);
};

const minimizeCloseClick = () => {
  minimize.value = false;
  handleClose(null, true);
};
</script>

<style lang="less" scoped>
.dia-footer {
  text-align: right;
  width: 100%;
  border-top: 1px solid #f1f1f1;
  text-align: right;
  padding: 6px 8px;
}
</style>

<style scoped lang="less">
.vol-dialog ::v-deep(.el-overlay-dialog) {
  display: flex !important;
}

.vol-dialog ::v-deep(.el-dialog) {
  margin: auto;
}

.vol-dialog ::v-deep(.el-dialog) {
  border-top-left-radius: 4px;
  border-top-right-radius: 4px;
}

.vol-dialog ::v-deep(.el-dialog__header) {
  border-top-left-radius: 4px;
  border-top-right-radius: 4px;
  padding: 0px 13px;
  line-height: 53px;
  border-bottom: 1px solid #f1f1f1;
  height: 50px;
  color: rgb(79, 79, 79);
  font-weight: bold;
  font-size: 14px;
  margin: 0;
  // background-image: linear-gradient(135deg, #0cd7bd 10%, #50c3f7);
}

.vol-dialog ::v-deep(.el-dialog__footer),
.vol-dialog ::v-deep(.el-dialog__body) {
  padding: 0;
}

.vol-dialog ::v-deep(.el-dialog__headerbtn) {
  top: 0;
  padding-top: 8px;
  height: 50px;
  width: 0;
  padding-right: 30px;
  padding-left: 5px;
}

// .vol-dialog ::v-deep(.el-dialog__headerbtn .el-dialog__close) {
//   color: #fff;
// }
.minimize {
  z-index: 9999;
  background: #ffff;
  min-width: 200px;
  display: flex;
  padding: 10px;
  font-size: 13px;
  bottom: 15px;
  right: 15px;
  border: 1px solid var(--el-notification-border-color);
  position: fixed;
  background-color: var(--el-bg-color-overlay);
  box-shadow: 0px 2px 8px 3px #eee;
  border-radius: 5px;
  align-items: center;

  .fx-1 {
    flex: 1;
    padding-right: 10px;

    i {
      margin-right: 3px;
    }
  }

  ::v-deep(button) {
    color: #000;
    padding: 0 !important;
    font-size: 14px !important;
    font-weight: bolder;
    height: 22px;
  }
}
</style>
