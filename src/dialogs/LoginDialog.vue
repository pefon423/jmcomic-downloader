<script setup lang="ts">
import { ref } from 'vue'
import { commands } from '../bindings.ts'
import { NButton, NCheckbox, NDialog, NModal, NTooltip, useMessage } from 'naive-ui'
import FloatLabelInput from '../components/FloatLabelInput.vue'
import { useStore } from '../store.ts'

const store = useStore()

const message = useMessage()

const showing = defineModel<boolean>('showing', { required: true })

const username = ref<string>(store.config?.username ?? '')
const password = ref<string>(store.config?.password ?? '')
const remember = ref<boolean>(username.value !== '' && password.value !== '')

async function onLogin() {
  if (store.config === undefined) {
    return
  }
  if (username.value === '') {
    message.error('請輸入使用者名')
    return
  }
  if (password.value === '') {
    message.error('請輸入密碼')
    return
  }

  const result = await commands.login(username.value, password.value)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  store.userProfile = result.data
  message.success('登入成功')
  if (remember.value) {
    store.config.username = username.value
    store.config.password = password.value
  }
  showing.value = false
}

function clearUsernameAndPasswordInConfig() {
  if (store.config === undefined) {
    return
  }

  store.config.username = ''
  store.config.password = ''
}
</script>

<template>
  <n-modal v-model:show="showing">
    <n-dialog
      :showIcon="false"
      title="賬號登入"
      positive-text="登入"
      @positive-click="onLogin"
      @close="showing = false"
      @keydown.enter="onLogin">
      <div class="flex flex-col gap-2">
        <FloatLabelInput label="使用者名" v-model:value="username" />
        <FloatLabelInput label="密碼" v-model:value="password" type="password" />
        <div class="flex justify-between">
          <n-tooltip>
            使用者名和密碼將以明文保存在配置檔案中
            <template #trigger>
              <n-checkbox v-model:checked="remember">記住我</n-checkbox>
            </template>
          </n-tooltip>
          <n-button type="primary" size="tiny" secondary @click="clearUsernameAndPasswordInConfig">
            清除配置檔案中的使用者名和密碼
          </n-button>
        </div>
      </div>
    </n-dialog>
  </n-modal>
</template>
=