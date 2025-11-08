import { Test, TestingModule } from '@nestjs/testing';
import { KindService } from './kind.service';

describe('KindService', () => {
  let service: KindService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [KindService],
    }).compile();

    service = module.get<KindService>(KindService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
