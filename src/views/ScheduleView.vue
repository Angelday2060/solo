<template>
  <div class="view-root">
    <div class="layout-schedule">
      <div class="panel schedule-list">
        <div class="panel-h">
          列表视图 · 标题 / 详情 / 起止 / 全天 / 提醒 / 重复（无+按周…）
        </div>
        <div class="panel-body">
          <div class="row head">
            <span>事项</span>
            <span>时间</span>
            <span>提醒</span>
            <span>重复</span>
          </div>
          <div class="row">
            <span class="placeholder" style="border: none; padding: 0">
              示例事项标题
            </span>
            <span>—</span>
            <span>系统通知</span>
            <span>无</span>
          </div>
        </div>
      </div>
      <div class="panel">
        <div class="panel-h">日历（增强 · 可选）</div>
        <div class="panel-body">
          <div class="placeholder" style="min-height: 200px">
            月历 / 周历 占位
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
