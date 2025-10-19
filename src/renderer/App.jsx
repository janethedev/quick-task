import { useState, useEffect } from 'react';
import { Layout, Flex, ConfigProvider } from 'antd';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useTranslation } from 'react-i18next';
import zhCN from 'antd/locale/zh_CN';
import enUS from 'antd/locale/en_US';
import TitleBar from './components/TitleBar';
import TodoInput from './components/TodoInput';
import TodoStats from './components/TodoStats';
import TodoList from './components/TodoList';
import { playCheckSound, playUncheckSound } from './soundEffects'; 

const { Content } = Layout;

function App() {
  const { t, i18n } = useTranslation();
  const [todos, setTodos] = useState([]);
  const [isImportant, setIsImportant] = useState(false);
  
  // 根据当前语言选择 Ant Design locale
  const antdLocale = i18n.language === 'zh-CN' ? zhCN : enUS;

  // 排序待办事项：已完成的总是在未完成的后面，未完成中重要的在前面，同分组内按 order 排序
  const sortTodos = (todosToSort) => {
    return [...todosToSort].sort((a, b) => {
      // 第一优先级：已完成的总是在未完成的后面
      if (a.completed !== b.completed) {
        return a.completed - b.completed;
      }
      
      // 第二优先级：未完成任务中，重要的在前面
      if (!a.completed && a.important !== b.important) {
        return b.important - a.important; // true(1) - false(0) = 1，重要的排前面
      }
      
      // 第三优先级：同状态、同重要性内按 order 排序
      return (a.order || 0) - (b.order || 0);
    });
  };

  // 初始化：加载已保存的待办事项
  useEffect(() => {
    const loadTodos = async () => {
      try {
        const loadedTodos = await window.electronAPI.loadTodos();
        
        // 向后兼容：为旧数据添加 important 和 order 字段
        const todosWithFields = loadedTodos.map((todo, index) => ({
          ...todo,
          important: todo.important || false,
          order: todo.order !== undefined ? todo.order : index
        }));
        
        setTodos(sortTodos(todosWithFields));
      } catch (error) {
        console.error(t('app.loadTodosFailed'), error);
        // 即使加载失败，也继续使用空数组
      } finally {
        // 无论成功还是失败，都显示窗口
        try {
          const appWindow = getCurrentWindow();
          await appWindow.show();
        } catch (err) {
          console.error(t('app.showWindowFailed'), err);
        }
      }
    };

    loadTodos();
  }, []);

  // 保存待办事项到文件
  const saveTodos = async (newTodos) => {
    await window.electronAPI.saveTodos(newTodos);
  };

  // 添加待办事项
  const handleAddTodo = (text) => {
    const newTodo = {
      id: Date.now(),
      text: text,
      completed: false,
      important: isImportant,
      order: -1 // 临时值，排序后会重新规范化
    };

    const newTodos = [...todos, newTodo];
    // 先排序，确保新任务在正确的位置（重要任务在前）
    const sortedTodos = sortTodos(newTodos);
    // 规范化 order 值，防止数值无限增长
    const normalizedTodos = normalizeOrders(sortedTodos);
    
    setTodos(normalizedTodos);
    saveTodos(normalizedTodos);
  };

  // 切换完成状态
  const handleToggleComplete = (id) => {
    const todo = todos.find(t => t.id === id);
    if (!todo) return;
    
    const newCompleted = !todo.completed;

    // 播放音效
    if (newCompleted) {
      playCheckSound(); // 完成任务时播放
    } else {
      playUncheckSound(); // 取消完成时播放（可选）
    }
    
    // 更新完成状态
    const newTodos = todos.map(t =>
      t.id === id ? { ...t, completed: newCompleted } : t
    );
    
    // 重新排序（已完成的会自动移到后面，未完成的会根据 important 排序）
    const sortedTodos = sortTodos(newTodos);
    // 规范化 order 值
    const normalizedTodos = normalizeOrders(sortedTodos);
    
    setTodos(normalizedTodos);
    saveTodos(normalizedTodos);
  };

  // 删除待办事项
  const handleDelete = (id) => {
    const newTodos = todos.filter(todo => todo.id !== id);
    setTodos(newTodos);
    saveTodos(newTodos);
  };

  // 编辑待办事项
  const handleEdit = (id, newText) => {
    const newTodos = todos.map(todo =>
      todo.id === id ? { ...todo, text: newText } : todo
    );
    setTodos(newTodos);
    saveTodos(newTodos);
  };

  // 清空已完成的待办事项
  const handleClearCompleted = () => {
    const newTodos = todos.filter(todo => !todo.completed);
    setTodos(newTodos);
    saveTodos(newTodos);
  };

  // 规范化 order 值：重新分配为连续整数
  const normalizeOrders = (todosToNormalize) => {
    return todosToNormalize.map((todo, index) => ({
      ...todo,
      order: index
    }));
  };

  // 处理拖拽排序
  const handleReorder = (reorderedTodos) => {
    // 拖拽后需要重新规范化 order 值，并确保排序逻辑一致
    const todosWithNewOrder = normalizeOrders(reorderedTodos);
    
    // 重新排序以确保分组逻辑（重要任务在前，已完成在后）
    const sortedTodos = sortTodos(todosWithNewOrder);
    
    setTodos(sortedTodos);
    saveTodos(sortedTodos);
  };

  // 统计信息
  const totalTodos = todos.length;
  const completedTodos = todos.filter(todo => todo.completed).length;

  return (
    <ConfigProvider locale={antdLocale}>
      <Layout className="app" style={{ height: '100vh' }}>
        <TitleBar />
        <Content className="container" style={{ padding: '10px 20px' }}>
          <Flex vertical gap={10} style={{ height: '100%' }}>
            <TodoInput
              onAddTodo={handleAddTodo}
              isImportant={isImportant}
              onToggleImportant={() => setIsImportant(!isImportant)}
            />
            <TodoStats 
              total={totalTodos} 
              completed={completedTodos} 
              onClearCompleted={handleClearCompleted}
            />
            <div style={{ 
              flex: 1, 
              overflow: 'auto',
              marginLeft: '-14px',
              marginRight: '-4px'
               }}>
              <TodoList
                todos={todos}
                onToggleComplete={handleToggleComplete}
                onDelete={handleDelete}
                onEdit={handleEdit}
                onReorder={handleReorder}
              />
            </div>
          </Flex>
        </Content>
      </Layout>
    </ConfigProvider>
  );
}

export default App;

