// src/features/task/dto/mod.rs

pub mod requests;
pub mod responses;

// Re-export commonly used types for convenience
pub use requests::{
    BatchCreateTaskDto, BatchDeleteTaskDto, BatchUpdateTaskDto, BatchUpdateTaskItemDto,
    CreateTaskDto, TaskFilterDto, UpdateTaskDto,
};

pub use responses::{
    BatchCreateResponseDto, BatchDeleteResponseDto, BatchUpdateResponseDto, PaginatedTasksDto,
    TaskDto, TaskResponse,
};
