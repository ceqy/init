# 视图数据 Proto 转换实现进度

## 概述

正在实现 mdm-material 服务的视图数据 Proto 转换功能，使视图扩展 RPC 方法完全可用。

## ✅ 已完成 (100%)

### PlantData 转换 (100%)
- ✅ `proto_to_plant_data()` - Proto MaterialPlantData 转换为 Domain PlantData
- ✅ `plant_data_to_proto()` - Domain PlantData 转换为 Proto MaterialPlantData
- ✅ 枚举转换辅助函数：
  - `proto_to_procurement_type()` - 处理 ProcurementType 枚举
  - `procurement_type_to_proto()`
  - `proto_to_plant_status()` - 处理 PlantMaterialStatus 枚举
  - `plant_status_to_proto()`
- ✅ 正确处理 Unspecified 枚举值
- ✅ 编译通过

**文件**: `src/api/proto_converters.rs` (行 132-231)

### StorageData 转换 (100%)
- ✅ `proto_to_storage_data()` - Proto MaterialStorageData 转换为 Domain StorageData
- ✅ `storage_data_to_proto()` - Domain StorageData 转换为 Proto MaterialStorageData
- ✅ 最简单的转换，无复杂类型
- ✅ 编译通过

**文件**: `src/api/proto_converters.rs` (行 233-263)

### QualityData 转换 (100%)
- ✅ `proto_to_quality_data()` - Proto MaterialQualityData 转换为 Domain QualityData
- ✅ `quality_data_to_proto()` - Domain QualityData 转换为 Proto MaterialQualityData
- ✅ 处理货架期相关字段
- ✅ 编译通过

**文件**: `src/api/proto_converters.rs` (行 265-299)

### SalesData 转换 (100%)
- ✅ `proto_to_sales_data()` - Proto MaterialSalesData 转换为 Domain SalesData
- ✅ `sales_data_to_proto()` - Domain SalesData 转换为 Proto MaterialSalesData
- ✅ `proto_to_sales_status()` - SalesMaterialStatus 枚举转换
- ✅ `sales_status_to_proto()`
- ✅ 编译通过

**文件**: `src/api/proto_converters.rs` (行 301-357)

### 通用辅助函数 (100%)
- ✅ `proto_to_money()` - Proto Money 转换为 Domain Money
- ✅ `money_to_proto()` - Domain Money 转换为 Proto Money
- ✅ `money_to_proto_from_option_ref()` - 处理 Option<&Money> 类型
- ✅ `proto_to_timestamp()` - Proto Timestamp 转换为 DateTime<Utc>
- ✅ `timestamp_to_proto()` - DateTime<Utc> 转换为 Proto Timestamp
- ✅ 编译通过

**文件**: `src/api/proto_converters.rs` (行 359-391, 455-462)

### AccountingData 转换 (100%)
- ✅ `proto_to_accounting_data()` - Proto MaterialAccountingData 转换为 Domain AccountingData
- ✅ `accounting_data_to_proto()` - Domain AccountingData 转换为 Proto MaterialAccountingData
- ✅ `proto_to_price_control()` - PriceControl 枚举转换
- ✅ `price_control_to_proto()`
- ✅ 处理 Optional Money 字段
- ✅ 编译通过

**文件**: `src/api/proto_converters.rs` (行 393-462)

### PurchaseData 转换 (100%)
- ✅ `proto_to_purchase_data()` - Proto MaterialPurchaseData 转换为 Domain PurchaseData
- ✅ `purchase_data_to_proto()` - Domain PurchaseData 转换为 Proto MaterialPurchaseData
- ✅ `proto_to_purchase_status()` - PurchaseMaterialStatus 枚举转换
- ✅ `purchase_status_to_proto()`
- ✅ 处理 Money 和 Timestamp 复杂类型
- ✅ 编译通过

**文件**: `src/api/proto_converters.rs` (行 464-549)

### gRPC Service 视图扩展方法 (100%)
- ✅ `extend_material_to_plant` - 扩展物料到工厂
- ✅ `extend_material_to_sales_org` - 扩展物料到销售组织
- ✅ `extend_material_to_purchase_org` - 扩展物料到采购组织
- ✅ `update_plant_data` - 更新工厂数据
- ✅ `update_sales_data` - 更新销售数据
- ✅ `update_purchase_data` - 更新采购数据
- ✅ 所有方法返回更新后的完整 Material 对象
- ✅ 编译通过

**文件**: `src/api/grpc_service.rs` (行 720-1000+)

## 实现统计

| 任务 | 复杂度 | 状态 |
|------|--------|------|
| PlantData | 中 | ✅ 完成 |
| StorageData | 低 | ✅ 完成 |
| QualityData | 低 | ✅ 完成 |
| SalesData | 中 | ✅ 完成 |
| 通用辅助函数 | 低 | ✅ 完成 |
| PurchaseData | 高 | ✅ 完成 |
| AccountingData | 中 | ✅ 完成 |
| 更新 gRPC Service | 中 | ✅ 完成 |
| **总计** | - | **100% 完成** |

## 技术要点

### 枚举转换模式
```rust
// Proto (i32) → Domain
fn proto_to_xxx_status(value: i32) -> XxxStatus {
    match value {
        0 => XxxStatus::Unspecified,
        1 => XxxStatus::Variant1,
        _ => XxxStatus::Unspecified, // 默认值
    }
}

// Domain → Proto
fn xxx_status_to_proto(value: XxxStatus) -> v1::XxxStatus {
    match value {
        XxxStatus::Unspecified => v1::XxxStatus::Unspecified,
        XxxStatus::Variant1 => v1::XxxStatus::Variant1,
        // 必须处理所有变体
    }
}
```

### Builder 模式使用
```rust
pub fn proto_to_xxx_data(proto: v1::MaterialXxxData) -> XxxData {
    XxxData::new(proto.key_field)
        .with_field1(proto.field1)
        .with_field2(proto.field2)
        // ... 链式调用
}
```

### Money 和 Timestamp 处理
```rust
// Money 转换
fn proto_to_money(proto: Option<common::v1::Money>) -> Option<Money> {
    proto.map(|m| Money::new(m.currency, m.amount, m.decimal_places))
}

// 处理 Option<&Money> 类型
fn money_to_proto_from_option_ref(money: Option<&Money>) -> Option<common::v1::Money> {
    money.map(|m| common::v1::Money {
        currency: m.currency.clone(),
        amount: m.amount,
        decimal_places: m.decimal_places,
    })
}

// Timestamp 转换
fn proto_to_timestamp(proto: Option<prost_types::Timestamp>) -> Option<DateTime<Utc>> {
    proto.and_then(|ts| DateTime::from_timestamp(ts.seconds, ts.nanos as u32))
}
```

### gRPC Service 方法模式
```rust
async fn extend_material_to_xxx(
    &self,
    request: Request<ExtendMaterialToXxxRequest>,
) -> Result<Response<ExtendMaterialToXxxResponse>, Status> {
    // 1. 提取认证信息
    let tenant_id = extract_tenant_id(metadata)?;
    let user_id = extract_user_id(metadata)?;

    // 2. 解析请求参数
    let material_id = parse_material_id(&req.material_id)?;
    let proto_data = req.xxx_data.ok_or_else(|| Status::invalid_argument("Missing data"))?;

    // 3. Proto → Domain 转换
    let domain_data = proto_to_xxx_data(proto_data);

    // 4. 创建并执行命令
    let cmd = ExtendToXxxCommand { ... };
    self.handler.extend_to_xxx(cmd).await?;

    // 5. 获取更新后的物料
    let query = GetMaterialQuery { material_id, tenant_id };
    let material = self.handler.get_material(query).await?;

    // 6. Domain → Proto 转换并返回
    Ok(Response::new(ExtendMaterialToXxxResponse {
        material: Some(material_to_proto(&material)),
    }))
}
```

### 注意事项
1. ✅ 所有枚举必须处理 Unspecified 变体
2. ✅ Proto 枚举值从 0 开始（0 = Unspecified）
3. ✅ 使用 Builder 模式构建 Domain 对象
4. ✅ Money 和 Timestamp 需要特殊处理
5. ✅ Optional 字段需要正确处理 None 情况
6. ✅ gRPC 方法返回完整的 Material 对象，需要重新查询

## 编译状态

✅ **编译成功** - 所有代码编译通过，无错误
⚠️ **15个警告** - 主要是未使用的变量和 Result 值，不影响功能

## 下一步建议

1. **添加单元测试** - 为所有转换函数添加测试
2. **添加集成测试** - 测试完整的 RPC 调用流程
3. **性能优化** - 考虑缓存和批量查询优化
4. **文档完善** - 添加 API 使用示例和最佳实践

## 参考资料

- Proto 定义: `/Users/x/init/proto/mdm/material/v1/material.proto`
- Domain 定义: `/Users/x/init/services/mdm-material/src/domain/views/`
- 转换实现: `/Users/x/init/services/mdm-material/src/api/proto_converters.rs`
- gRPC Service: `/Users/x/init/services/mdm-material/src/api/grpc_service.rs`
**需要实现**:
- `proto_to_sales_data()` - Proto MaterialSalesData → Domain SalesData
- `sales_data_to_proto()` - Domain SalesData → Proto MaterialSalesData
- `proto_to_sales_status()` - SalesMaterialStatus 枚举转换
- `sales_status_to_proto()`

**字段映射**:
- sales_org, distribution_channel, division
- sales_unit, minimum_order_quantity, minimum_delivery_quantity
- delivery_unit, delivery_days
- pricing_reference_material, material_pricing_group
- account_assignment_group, tax_classification
- availability_check, status, deletion_flag

### PurchaseData 转换 (0%)
**需要实现**:
- `proto_to_purchase_data()` - Proto MaterialPurchaseData → Domain PurchaseData
- `purchase_data_to_proto()` - Domain PurchaseData → Proto MaterialPurchaseData
- `proto_to_purchase_status()` - PurchaseMaterialStatus 枚举转换
- `purchase_status_to_proto()`
- Money 类型转换辅助函数
- Timestamp 类型转换辅助函数

**字段映射**:
- purchase_org, plant, purchase_unit
- order_unit_conversion, purchasing_group
- planned_delivery_days, over_delivery_tolerance, under_delivery_tolerance
- unlimited_over_delivery, preferred_vendor_id
- standard_price, last_purchase_price, last_purchase_date
- automatic_po, source_list, status, deletion_flag

### StorageData 转换 (0%)
**需要实现**:
- `proto_to_storage_data()` - Proto MaterialStorageData → Domain StorageData
- `storage_data_to_proto()` - Domain StorageData → Proto MaterialStorageData

**字段映射**:
- plant, storage_location, warehouse_number
- storage_type, storage_bin
- max_storage_quantity, min_storage_quantity
- storage_unit_type, picking_area
- storage_section, deletion_flag

### AccountingData 转换 (0%)
**需要实现**:
- `proto_to_accounting_data()` - Proto MaterialAccountingData → Domain AccountingData
- `accounting_data_to_proto()` - Domain MaterialAccountingData → Proto MaterialAccountingData
- `proto_to_price_control()` - PriceControl 枚举转换
- `price_control_to_proto()`
- Money 类型转换辅助函数（如果 PurchaseData 未实现）

**字段映射**:
- plant, valuation_area, valuation_class
- price_control, standard_price, moving_average_price
- price_unit, price_unit_quantity
- inventory_account, price_difference_account, cost_element
- costing_lot_size, with_qty_structure, deletion_flag

### QualityData 转换 (0%)
**需要实现**:
- `proto_to_quality_data()` - Proto MaterialQualityData → Domain QualityData
- `quality_data_to_proto()` - Domain MaterialQualityData → Proto MaterialQualityData

**字段映射**:
- plant, inspection_active, inspection_type
- inspection_plan, sample_percentage
- certificate_required, quality_management_control_key
- shelf_life_days, remaining_shelf_life, total_shelf_life
- deletion_flag

## 通用辅助函数需求

### Money 类型转换
```rust
fn proto_to_money(proto: Option<common::v1::Money>) -> Option<Money>
fn money_to_proto(money: &Option<Money>) -> Option<common::v1::Money>
```

### Timestamp 类型转换
```rust
fn proto_to_timestamp(proto: Option<prost_types::Timestamp>) -> Option<chrono::DateTime<Utc>>
fn timestamp_to_proto(dt: &Option<chrono::DateTime<Utc>>) -> Option<prost_types::Timestamp>
```

## 实现策略

### 第一步：完成所有简单视图数据转换
1. ✅ PlantData (已完成)
2. StorageData (最简单，无复杂类型)
3. QualityData (简单，无复杂类型)
4. SalesData (中等复杂度)

### 第二步：实现通用辅助函数
1. Money 类型转换
2. Timestamp 类型转换

### 第三步：完成复杂视图数据转换
1. PurchaseData (需要 Money 和 Timestamp)
2. AccountingData (需要 Money 和 PriceControl)

### 第四步：更新 gRPC Service
完成所有转换后，更新 `src/api/grpc_service.rs` 中的 6 个视图扩展方法：
- extend_material_to_plant
- extend_material_to_sales_org
- extend_material_to_purchase_org
- update_plant_data
- update_sales_data
- update_purchase_data

## 预计工作量

| 任务 | 复杂度 | 预计时间 | 状态 |
|------|--------|----------|------|
| PlantData | 中 | 1-2小时 | ✅ 完成 |
| StorageData | 低 | 30分钟 | ⏳ 待实现 |
| QualityData | 低 | 30分钟 | ⏳ 待实现 |
| SalesData | 中 | 1小时 | ⏳ 待实现 |
| 通用辅助函数 | 低 | 30分钟 | ⏳ 待实现 |
| PurchaseData | 高 | 1-2小时 | ⏳ 待实现 |
| AccountingData | 中 | 1小时 | ⏳ 待实现 |
| 更新 gRPC Service | 中 | 1小时 | ⏳ 待实现 |
| **总计** | - | **6-8小时** | **12.5% 完成** |

## 下一步行动

1. **立即开始**: 实现 StorageData 转换（最简单）
2. **然后**: 实现 QualityData 转换
3. **接着**: 实现 SalesData 转换
4. **之后**: 实现通用辅助函数（Money, Timestamp）
5. **最后**: 完成 PurchaseData 和 AccountingData

## 技术要点

### 枚举转换模式
```rust
// Proto (i32) → Domain
fn proto_to_xxx_status(value: i32) -> XxxStatus {
    match value {
        0 => XxxStatus::Unspecified,
        1 => XxxStatus::Variant1,
        _ => XxxStatus::Unspecified, // 默认值
    }
}

// Domain → Proto
fn xxx_status_to_proto(value: XxxStatus) -> v1::XxxStatus {
    match value {
        XxxStatus::Unspecified => v1::XxxStatus::Unspecified,
        XxxStatus::Variant1 => v1::XxxStatus::Variant1,
        // 必须处理所有变体
    }
}
```

### Builder 模式使用
```rust
pub fn proto_to_xxx_data(proto: v1::MaterialXxxData) -> XxxData {
    XxxData::new(proto.key_field)
        .with_field1(proto.field1)
        .with_field2(proto.field2)
        // ... 链式调用
}
```

### 注意事项
1. ✅ 所有枚举必须处理 Unspecified 变体
2. ✅ Proto 枚举值从 0 开始（0 = Unspecified）
3. ✅ 使用 Builder 模式构建 Domain 对象
4. ⚠️ Money 和 Timestamp 需要特殊处理
5. ⚠️ Optional 字段需要正确处理 None 情况

## 测试计划

完成所有转换后，需要添加单元测试：
- [ ] PlantData 双向转换测试
- [ ] SalesData 双向转换测试
- [ ] PurchaseData 双向转换测试
- [ ] StorageData 双向转换测试
- [ ] AccountingData 双向转换测试
- [ ] QualityData 双向转换测试
- [ ] 枚举转换边界测试
- [ ] Optional 字段测试

## 参考资料

- Proto 定义: `/Users/x/init/proto/mdm/material/v1/material.proto`
- Domain 定义: `/Users/x/init/services/mdm-material/src/domain/views/`
- 转换实现: `/Users/x/init/services/mdm-material/src/api/proto_converters.rs`
- gRPC Service: `/Users/x/init/services/mdm-material/src/api/grpc_service.rs`
